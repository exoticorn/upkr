use std::collections::HashMap;
use std::rc::Rc;

use crate::lz;
use crate::match_finder::MatchFinder;
use crate::rans::{CostCounter, RansCoder};

pub fn pack(data: &[u8]) -> Vec<u8> {
    let mut parse = parse(data);
    let mut ops = vec![];
    while let Some(link) = parse {
        ops.push(link.op);
        parse = link.prev.clone();
    }
    let mut state = lz::CoderState::new();
    let mut coder = RansCoder::new();
    for op in ops.into_iter().rev() {
        op.encode(&mut coder, &mut state);
    }
    lz::encode_eof(&mut coder, &mut state);
    coder.finish()
}

struct Parse {
    prev: Option<Rc<Parse>>,
    op: lz::Op,
}

struct Arrival {
    parse: Option<Rc<Parse>>,
    state: lz::CoderState,
    cost: f64,
}

type Arrivals = HashMap<usize, Vec<Arrival>>;

const MAX_ARRIVALS: usize = 4;

fn parse(data: &[u8]) -> Option<Rc<Parse>> {
    let match_finder = MatchFinder::new(data);
    let mut near_matches = [usize::MAX; 1024];
    let mut last_seen = [usize::MAX; 256];

    let mut arrivals: Arrivals = HashMap::new();
    fn add_arrival(arrivals: &mut Arrivals, pos: usize, arrival: Arrival) {
        let vec = arrivals.entry(pos).or_default();
        if vec.len() < MAX_ARRIVALS || vec[MAX_ARRIVALS - 1].cost > arrival.cost {
            vec.push(arrival);
            vec.sort_by(|a, b| {
                a.cost
                    .partial_cmp(&b.cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if vec.len() > MAX_ARRIVALS {
                vec.pop();
            }
        }
    }
    fn add_match(
        arrivals: &mut Arrivals,
        pos: usize,
        offset: usize,
        length: usize,
        arrival: &Arrival,
    ) {
        let mut cost_counter = CostCounter(0.);
        let mut state = arrival.state.clone();
        let op = lz::Op::Match {
            offset: offset as u32,
            len: length as u32,
        };
        op.encode(&mut cost_counter, &mut state);
        add_arrival(
            arrivals,
            pos + length,
            Arrival {
                parse: Some(Rc::new(Parse {
                    prev: arrival.parse.clone(),
                    op,
                })),
                state,
                cost: arrival.cost + cost_counter.0,
            },
        );
    }
    add_arrival(
        &mut arrivals,
        0,
        Arrival {
            parse: None,
            state: lz::CoderState::new(),
            cost: 0.0,
        },
    );
    let mut best_per_offset = HashMap::new();
    for pos in 0..data.len() {
        let match_length = |offset: usize| {
            data[pos..]
                .iter()
                .zip(data[(pos - offset)..].iter())
                .take_while(|(a, b)| a == b)
                .count()
        };

        let here_arrivals = if let Some(arr) = arrivals.remove(&pos) {
            arr
        } else {
            continue;
        };
        best_per_offset.clear();
        let mut best_cost = f64::MAX;
        for arrival in &here_arrivals {
            best_cost = best_cost.min(arrival.cost);
            let per_offset = best_per_offset
                .entry(arrival.state.last_offset())
                .or_insert(f64::MAX);
            *per_offset = per_offset.min(arrival.cost);
        }

        for arrival in here_arrivals {
            if arrival.cost > (best_cost + 32.0).min(*best_per_offset.get(&arrival.state.last_offset()).unwrap()) {
                continue;
            }
            let mut found_last_offset = false;
            let mut closest_match = None;
            for m in match_finder.matches(pos) {
                closest_match = Some(closest_match.unwrap_or(0).max(m.pos));
                let offset = pos - m.pos;
                found_last_offset |= offset as u32 == arrival.state.last_offset();
                add_match(&mut arrivals, pos, offset, m.length, &arrival);
            }

            let mut near_matches_left = 4;
            let mut match_pos = last_seen[data[pos] as usize];
            while near_matches_left > 0
                && match_pos != usize::MAX
                && closest_match.iter().all(|p| *p < match_pos)
            {
                let offset = pos - match_pos;
                let length = match_length(offset);
                assert!(length > 0);
                add_match(&mut arrivals, pos, offset, length, &arrival);
                found_last_offset |= offset as u32 == arrival.state.last_offset();
                if offset < near_matches.len() {
                    match_pos = near_matches[match_pos % near_matches.len()];
                }
                near_matches_left -= 1;
            }

            if !found_last_offset && arrival.state.last_offset() > 0 {
                let offset = arrival.state.last_offset() as usize;
                let length = match_length(offset);
                if length > 0 {
                    add_match(&mut arrivals, pos, offset, length, &arrival);
                }
            }

            let mut cost_counter = CostCounter(0.);
            let mut state = arrival.state;
            let op = lz::Op::Literal(data[pos]);
            op.encode(&mut cost_counter, &mut state);
            add_arrival(
                &mut arrivals,
                pos + 1,
                Arrival {
                    parse: Some(Rc::new(Parse {
                        prev: arrival.parse,
                        op,
                    })),
                    state,
                    cost: arrival.cost + cost_counter.0,
                },
            );
        }
        near_matches[pos % near_matches.len()] = last_seen[data[pos] as usize];
        last_seen[data[pos] as usize] = pos;
    }
    arrivals.remove(&data.len()).unwrap()[0].parse.clone()
}

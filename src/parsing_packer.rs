use std::collections::HashMap;
use std::rc::Rc;

use crate::match_finder::MatchFinder;
use crate::rans::{RansCoder, CostCounter};
use crate::lz;

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

const MAX_ARRIVALS: usize = 16;

fn parse(data: &[u8]) -> Option<Rc<Parse>> {
    let match_finder = MatchFinder::new(data);

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
    fn add_match(arrivals: &mut Arrivals, pos: usize, offset: usize, length: usize, arrival: &Arrival) {
        let mut cost_counter = CostCounter(0.);
        let mut state = arrival.state.clone();
        let op = lz::Op::Match { offset: offset as u32, len: length as u32 };
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
    for pos in 0..data.len() {
        for arrival in arrivals.remove(&pos).unwrap() {
            let mut found_last_offset = false;
            for m in match_finder.matches(pos) {
                let offset = pos - m.pos;
                if offset as u32 == arrival.state.last_offset() {
                    found_last_offset = true;
                }
                add_match(&mut arrivals, pos, offset, m.length, &arrival);
            }

            if !found_last_offset && arrival.state.last_offset() > 0 {
                let offset = arrival.state.last_offset() as usize;
                let length = data[pos..].iter().zip(data[(pos - offset)..].iter()).take_while(|(a, b)| a == b).count();
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
    }
    arrivals.remove(&data.len()).unwrap()[0].parse.clone()
}

use std::collections::{HashMap, HashSet};
use std::mem;
use std::rc::Rc;

use crate::match_finder::MatchFinder;
use crate::rans::{CostCounter, RansCoder};
use crate::{lz, ProgressCallback};

pub fn pack(
    data: &[u8],
    level: u8,
    config: &crate::Config,
    progress_cb: Option<ProgressCallback>,
) -> Vec<u8> {
    let mut parse = parse(data, Config::from_level(level), config, progress_cb);
    let mut ops = vec![];
    while let Some(link) = parse {
        ops.push(link.op);
        parse = link.prev.clone();
    }
    let mut state = lz::CoderState::new(config);
    let mut coder = RansCoder::new(config);
    for op in ops.into_iter().rev() {
        op.encode(&mut coder, &mut state, config);
    }
    lz::encode_eof(&mut coder, &mut state, config);
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

fn parse(
    data: &[u8],
    config: Config,
    encoding_config: &crate::Config,
    mut progress_cb: Option<ProgressCallback>,
) -> Option<Rc<Parse>> {
    let mut match_finder = MatchFinder::new(data)
        .with_max_queue_size(config.max_queue_size)
        .with_patience(config.patience)
        .with_max_matches_per_length(config.max_matches_per_length)
        .with_max_length_diff(config.max_length_diff);
    let mut near_matches = [usize::MAX; 1024];
    let mut last_seen = [usize::MAX; 256];

    let max_arrivals = config.max_arrivals;

    let mut arrivals: Arrivals = HashMap::new();
    fn sort_arrivals(vec: &mut Vec<Arrival>, max_arrivals: usize) {
        if max_arrivals == 0 {
            return;
        }
        vec.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut seen_offsets = HashSet::new();
        let mut remaining = Vec::new();
        for arr in mem::replace(vec, Vec::new()) {
            if seen_offsets.insert(arr.state.last_offset()) {
                if vec.len() < max_arrivals {
                    vec.push(arr);
                }
            } else {
                remaining.push(arr);
            }
        }
        for arr in remaining {
            if vec.len() >= max_arrivals {
                break;
            }
            vec.push(arr);
        }
    }

    fn add_arrival(arrivals: &mut Arrivals, pos: usize, arrival: Arrival, max_arrivals: usize) {
        let vec = arrivals.entry(pos).or_default();
        if max_arrivals == 0 {
            if vec.is_empty() {
                vec.push(arrival);
            } else if vec[0].cost > arrival.cost {
                vec[0] = arrival;
            }
            return;
        }
        vec.push(arrival);
        if vec.len() > max_arrivals * 2 {
            sort_arrivals(vec, max_arrivals);
        }
    }
    fn add_match(
        arrivals: &mut Arrivals,
        cost_counter: &mut CostCounter,
        pos: usize,
        offset: usize,
        mut length: usize,
        arrival: &Arrival,
        max_arrivals: usize,
        config: &crate::Config,
    ) {
        if length < config.min_length() {
            return;
        }
        length = length.min(config.max_length);
        cost_counter.reset();
        let mut state = arrival.state.clone();
        let op = lz::Op::Match {
            offset: offset as u32,
            len: length as u32,
        };
        op.encode(cost_counter, &mut state, config);
        add_arrival(
            arrivals,
            pos + length,
            Arrival {
                parse: Some(Rc::new(Parse {
                    prev: arrival.parse.clone(),
                    op,
                })),
                state,
                cost: arrival.cost + cost_counter.cost(),
            },
            max_arrivals,
        );
    }
    add_arrival(
        &mut arrivals,
        0,
        Arrival {
            parse: None,
            state: lz::CoderState::new(encoding_config),
            cost: 0.0,
        },
        max_arrivals,
    );

    let cost_counter = &mut CostCounter::new(encoding_config);
    let mut best_per_offset = HashMap::new();
    for pos in 0..data.len() {
        let match_length = |offset: usize| {
            data[pos..]
                .iter()
                .zip(data[(pos - offset)..].iter())
                .take_while(|(a, b)| a == b)
                .count()
        };

        let here_arrivals = if let Some(mut arr) = arrivals.remove(&pos) {
            sort_arrivals(&mut arr, max_arrivals);
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

        'arrival_loop: for arrival in here_arrivals {
            if arrival.cost
                > (best_cost + config.max_cost_delta).min(
                    *best_per_offset.get(&arrival.state.last_offset()).unwrap()
                        + config.max_offset_cost_delta,
                )
            {
                continue;
            }
            let mut found_last_offset = false;
            let mut closest_match = None;
            for m in match_finder.matches(pos) {
                closest_match = Some(closest_match.unwrap_or(0).max(m.pos));
                let offset = pos - m.pos;
                if offset <= encoding_config.max_offset {
                    found_last_offset |= offset as u32 == arrival.state.last_offset();
                    add_match(
                        &mut arrivals,
                        cost_counter,
                        pos,
                        offset,
                        m.length,
                        &arrival,
                        max_arrivals,
                        encoding_config,
                    );
                    if m.length >= config.greedy_size {
                        break 'arrival_loop;
                    }
                }
            }

            let mut near_matches_left = config.num_near_matches;
            let mut match_pos = last_seen[data[pos] as usize];
            while near_matches_left > 0
                && match_pos != usize::MAX
                && closest_match.iter().all(|p| *p < match_pos)
            {
                let offset = pos - match_pos;
                if offset > encoding_config.max_offset {
                    break;
                }
                let length = match_length(offset);
                assert!(length > 0);
                add_match(
                    &mut arrivals,
                    cost_counter,
                    pos,
                    offset,
                    length,
                    &arrival,
                    max_arrivals,
                    encoding_config,
                );
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
                    add_match(
                        &mut arrivals,
                        cost_counter,
                        pos,
                        offset,
                        length,
                        &arrival,
                        max_arrivals,
                        encoding_config,
                    );
                }
            }

            cost_counter.reset();
            let mut state = arrival.state;
            let op = lz::Op::Literal(data[pos]);
            op.encode(cost_counter, &mut state, encoding_config);
            add_arrival(
                &mut arrivals,
                pos + 1,
                Arrival {
                    parse: Some(Rc::new(Parse {
                        prev: arrival.parse,
                        op,
                    })),
                    state,
                    cost: arrival.cost + cost_counter.cost(),
                },
                max_arrivals,
            );
        }
        near_matches[pos % near_matches.len()] = last_seen[data[pos] as usize];
        last_seen[data[pos] as usize] = pos;
        if let Some(ref mut cb) = progress_cb {
            cb(pos + 1);
        }
    }
    arrivals.remove(&data.len()).unwrap()[0].parse.clone()
}

struct Config {
    max_arrivals: usize,
    max_cost_delta: f64,
    max_offset_cost_delta: f64,
    num_near_matches: usize,
    greedy_size: usize,
    max_queue_size: usize,
    patience: usize,
    max_matches_per_length: usize,
    max_length_diff: usize,
}

impl Config {
    fn from_level(level: u8) -> Config {
        let max_arrivals = match level {
            0..=1 => 0,
            2 => 2,
            3 => 4,
            4 => 8,
            5 => 16,
            6 => 32,
            7 => 64,
            8 => 96,
            _ => 128,
        };
        let (max_cost_delta, max_offset_cost_delta) = match level {
            0..=4 => (16.0, 0.0),
            5..=8 => (16.0, 4.0),
            _ => (16.0, 8.0),
        };
        let num_near_matches = level.saturating_sub(1) as usize;
        let greedy_size = 4 + level as usize * level as usize * 3;
        let max_length_diff = match level {
            0..=1 => 0,
            2..=3 => 1,
            4..=5 => 2,
            6..=7 => 3,
            _ => 4,
        };
        Config {
            max_arrivals,
            max_cost_delta,
            max_offset_cost_delta,
            num_near_matches,
            greedy_size,
            max_queue_size: level as usize * 100,
            patience: level as usize * 100,
            max_matches_per_length: level as usize,
            max_length_diff,
        }
    }
}

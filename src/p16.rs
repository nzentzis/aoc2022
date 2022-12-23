use anyhow::Result;

use std::collections::VecDeque;

#[derive(Debug)]
struct Valve {
    flow: usize,
    neighbors: Vec<usize>,
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    struct ValveLine {
        name: String,
        flow: usize,
        neighbors: Vec<String>,
    }

    let lines = crate::util::read_lines_regex(
        input,
        r#"^Valve (..) has flow rate=(\d+); tunnels? leads? to valves? (.+)$"#,
        |caps| {
            let name = caps.get(1).unwrap().as_str().to_owned();
            let flow = caps.get(2).unwrap().as_str().parse()?;
            let neighbors = caps.get(3).unwrap()
                           .as_str().split(", ")
                           .map(|s| s.to_owned())
                           .collect();
            Ok(ValveLine {name, flow, neighbors})
        }
    )?;

    // resolve names to indices
    let name_map = lines.iter().enumerate()
                  .map(|(idx, line)| (line.name.clone(), idx))
                  .collect::<std::collections::HashMap<_,_>>();
    let start = name_map.get("AA").ok_or_else(|| anyhow::anyhow!("No valve AA"))?;

    lines.into_iter()
         .map(|line| Ok(Valve {
             flow: line.flow,
             neighbors: line.neighbors.into_iter()
                       .map(|n| name_map.get(&n)
                               .cloned()
                               .ok_or_else(|| anyhow::anyhow!("No such valve: {}", n)))
                       .collect::<Result<Vec<_>>>()?
         }))
         .collect::<Result<Vec<_>>>()
         .map(|v| (*start, v))
}

/// Compute all-pairs shortest path via Floyd-Warshall
///
/// In the result array, `arr[i][j]` represents the shortest path (in timesteps) from valve `i` to
/// valve `j`.
fn all_pairs(input: &[Valve]) -> Vec<Vec<usize>> {
    let mut output = vec![vec![usize::MAX; input.len()]; input.len()];
    for (k, valve) in input.iter().enumerate() {
        output[k][k] = 0;
        for n in &valve.neighbors {
            output[k][*n] = 1;
        }
    }

    // relax lengths
    for k in 0..input.len() {
        for i in 0..input.len() {
            for j in 0..input.len() {
                output[i][j] = output[i][j].min(output[i][k].saturating_add(output[k][j]));
            }
        }
    }

    output
}

/// Compute the maximum flow achievable by visiting the given subset of all nodes
fn max_for_subset(input: &Input, apsp: &[Vec<usize>], subset: u64, t_max: usize) -> Result<(usize, usize)> {
    #[derive(Copy, Clone)]
    struct Partial {
        /// Set of untouched valves
        ///
        /// Bit `k` in this word being set represents the valve `input[k]` being available.
        avail: u64,

        /// Cumulative flow from available valves
        cum_flow: usize,

        /// Current score
        score: usize,

        /// Current valve
        pos: usize,

        /// Time remaining
        t: usize,
    }

    impl Partial {
        fn is_avail(&self, idx: usize) -> bool {
            self.avail & (1 << idx) != 0
        }

        fn lower_bound(&self, _input: &[Valve]) -> usize {
            self.score
        }

        fn upper_bound(&self, paths: &[Vec<usize>], valves: &[Valve]) -> usize {
            let mut s = self.score;
            let mut bits = self.avail;
            let paths = &paths[self.pos];
            for i in 0..(64-bits.leading_zeros()) as usize {
                if bits & 1 != 0 {
                    s += self.t.saturating_sub(paths[i] + 1) * valves[i].flow;
                }
                bits >>= 1;
            }

            s
        }

        fn branch<'p>(
            &'p self,
            paths: &'p [Vec<usize>],
            valves: &'p [Valve],
        ) -> impl Iterator<Item=Self> + 'p {
            (0..valves.len()).into_iter()
                             .filter(|idx| (paths[self.pos][*idx] + 1) < self.t && // reachable?
                                           self.is_avail(*idx)) // not already set?
                             .map(move |idx| {
                                 let start_flow_t = self.t - (paths[self.pos][idx] + 1);
                                 let score_bonus = valves[idx].flow * start_flow_t;
                                 Self {
                                     avail: self.avail ^ (1 << idx),
                                     score: self.score + score_bonus,
                                     cum_flow: self.cum_flow - valves[idx].flow,
                                     t: start_flow_t,
                                     pos: idx,
                                 }
                             })
        }
    }

    let valves = &input.1;
    let all_paths = apsp;

    let mut paths = VecDeque::new();
    paths.push_back(Partial {
        cum_flow: valves.iter().map(|v| v.flow).sum(),
        avail: subset,
        score: 0,
        t: t_max,
        pos: input.0,
    });

    let mut best_lower = 0;
    let mut considered = 0;
    while let Some(state) = if paths.len() < 1_000_000_000 { paths.pop_front() }
                            else { paths.pop_back() } {
        if state.lower_bound(valves) > best_lower {
            best_lower = state.lower_bound(valves);
            paths.retain(|st| st.upper_bound(all_paths, valves) > best_lower);
        }
        considered += 1;

        paths.extend(state.branch(all_paths, valves)
                          .filter(|st| st.upper_bound(all_paths, valves) > best_lower));
    }

    Ok((best_lower, considered))
}

fn solve1(input: &Input) -> Result<usize> {
    // preprocess to remove irrelevant valves
    let valves = &input.1;
    let mut flow_mask = 0;
    for (i,_) in valves.iter().enumerate().filter(|(_, v)| v.flow != 0) {
        flow_mask |= 1 << i;
    }

    let apsp = all_pairs(valves);
    let (best, _states) = max_for_subset(input, &apsp, flow_mask, 30)?;
    //println!("{}", states);
    Ok(best)
}

fn solve2(input: &Input) -> Result<usize> {
    use rayon::prelude::*;

    let valves = &input.1;
    let relevant_valves = valves.iter().enumerate()
                         .filter(|(_, v)| v.flow != 0)
                         .map(|t| t.0)
                         .collect::<Vec<_>>();
    anyhow::ensure!(relevant_valves.len() < 16,
                    "Too many relevant valves - you need a different algorithm");

    let n_max = (1 << relevant_valves.len()) - 1;
    let apsp = all_pairs(valves);

    let mut flow_mask = 0;
    for (i,_) in valves.iter().enumerate().filter(|(_, v)| v.flow != 0) {
        flow_mask |= 1 << i;
    }

    let states = std::sync::atomic::AtomicUsize::new(0);
    let res = (0..n_max).into_par_iter()
             .map(|mut mask| {
                 // we take items with 1 bits, they take items with 0 bits
                 let mut us = 0;
                 for v in relevant_valves.iter().cloned() {
                     if mask & 1 == 1 {
                         us |= 1 << v;
                     }
                     mask >>= 1;
                 }
                 let them = flow_mask ^ us;

                 let (us_score, us_st) = max_for_subset(input, &apsp, us, 26).unwrap();
                 let (them_score, them_st) = max_for_subset(input, &apsp, them, 26).unwrap();
                 states.fetch_add(us_st, std::sync::atomic::Ordering::Relaxed);
                 states.fetch_add(them_st, std::sync::atomic::Ordering::Relaxed);

                 us_score + them_score
             })
             .max().unwrap();
    //println!("{}", states.into_inner());
    Ok(res)
}

problem!(load_input => (usize, Vec<Valve>) => (solve1, solve2));

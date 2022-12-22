use anyhow::Result;
use rayon::prelude::*;

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    const PAT: &str =
        r#"^Blueprint (\d+): Each ore robot costs (\d+) ore. Each clay robot costs (\d+) ore. Each obsidian robot costs (\d+) ore and (\d+) clay. Each geode robot costs (\d+) ore and (\d+) obsidian.$"#;
    crate::util::read_lines_regex(input, PAT, |caps| Ok(Blueprint {
        index: caps.get(1).unwrap().as_str().parse()?,
        b_cost: caps.get(2).unwrap().as_str().parse()?,
        c_cost: caps.get(3).unwrap().as_str().parse()?,
        o_cost: (caps.get(4).unwrap().as_str().parse()?,
                 caps.get(5).unwrap().as_str().parse()?),
        g_cost: (caps.get(6).unwrap().as_str().parse()?,
                 caps.get(7).unwrap().as_str().parse()?),
    }))
}

struct Blueprint {
    /// Blueprint number
    index: usize,

    /// Cost of an ore robot, in ore
    b_cost: u8,

    /// Cost of a clay robot, in ore
    c_cost: u8,

    /// Cost of an obsidian robot, in (ore, clay)
    o_cost: (u8, u8),

    /// Cost of a geode robot, in (ore, obsidian)
    g_cost: (u8, u8),
}

/// Compute quality score for the given blueprint
fn max_geodes(bp: &Blueprint, t_max: usize) -> usize {
    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    #[repr(transparent)]
    struct State {
        key: u64,
    }

    impl State {
        const BOTS_MASK: u64 = 0x00_ffffff_000000;
        const TIME_MASK: u64 = 0xff_000000_000000;
        const TIMESTEP: u64  = 0x01_000000_000000;

        const ORE_MASK: u64  = 0x00_000000_ff0000;
        const CLAY_MASK: u64 = 0x00_000000_00ff00;
        const OBS_MASK: u64  = 0x00_000000_0000ff;

        const ORE_BOT_MASK: u64   = 0x00_ff0000_000000;
        const CLAY_BOT_MASK: u64  = 0x00_00ff00_000000;
        const OBS_BOT_MASK: u64   = 0x00_0000ff_000000;

        const ORE_BOT: u64   = 0x00_010000_000000;
        const CLAY_BOT: u64  = 0x00_000100_000000;
        const OBS_BOT: u64   = 0x00_000001_000000;

        /// Return a version of this state that advances time forwards by one tick
        ///
        /// This updates resource accumulation, but doesn't do anything else
        fn step(self) -> Self {
            let k0 = self.key;
            let k1 = k0 + ((k0 & Self::BOTS_MASK) >> 24);
            let k2 = k1 - Self::TIMESTEP;
            //println!("{:08x} {:08x} {:08x}", k0, k1, k2);

            Self { key: k2 }
        }

        fn has_time(self) -> bool {
            // no point doing anything if we have zero or one timesteps left
            (self.key & Self::TIME_MASK) > Self::TIMESTEP
        }

        fn remaining(self) -> usize {
            ((self.key >> 48) & 0xff) as usize
        }
    }

    struct Costs {
        b_ore: u32,
        c_ore: u32,
        o_ore: u32,
        o_clay: u32,
        g_ore: u32,
        g_obs: u32,

        /// Maximum ore that it makes sense to retain
        ///
        /// Makes no sense to go over the maximum ore required for a robot build per tick, so
        /// disable ore-robot builds once we hit this much. This is shifted for comparison with the
        /// ore robot count.
        max_ore: u64,

        /// Maximum clay that it makes sense to retain
        max_clay: u64,

        /// Maximum obsidian that it makes sense to retain
        max_obs: u64,
    }

    impl Costs {
        fn new(bp: &Blueprint) -> Self {
            Self {
                b_ore:  (bp.b_cost as u32) << 16,
                c_ore:  (bp.c_cost as u32) << 16,
                o_ore:  (bp.o_cost.0 as u32) << 16,
                g_ore:  (bp.g_cost.0 as u32) << 16,

                o_clay: (bp.o_cost.1 as u32) << 8,
                g_obs:  (bp.g_cost.1 as u32) << 0,

                max_ore: (bp.b_cost.max(bp.c_cost).max(bp.o_cost.0).max(bp.g_cost.0) as u64) << 40,
                max_clay: (bp.o_cost.1 as u64) << 32,
                max_obs: (bp.g_cost.1 as u64) << 24,
            }
        }

        fn can_ore(&self, s: State) -> bool {
            (s.key & State::ORE_MASK) as u32 >= self.b_ore &&
                (s.key & State::ORE_BOT_MASK) < self.max_ore
        }

        fn can_clay(&self, s: State) -> bool {
            (s.key & State::ORE_MASK) as u32 >= self.c_ore &&
                (s.key & State::CLAY_BOT_MASK) < self.max_clay

        }

        fn can_obs(&self, s: State) -> bool {
            (s.key & State::ORE_MASK) as u32 >= self.o_ore &&
                (s.key & State::CLAY_MASK) as u32 >= self.o_clay &&
                (s.key & State::OBS_BOT_MASK) < self.max_obs

        }

        fn can_geo(&self, s: State) -> bool {
            (s.key & State::ORE_MASK) as u32 >= self.g_ore &&
                (s.key & State::OBS_MASK) as u32 >= self.g_obs
        }
    }

    /// Compute maximum number of geodes achievable from provided state
    fn f(
        costs: &Costs,
        store: &mut fnv::FnvHashMap<State, usize>,
        x: State
    ) -> usize {
        if !x.has_time() {
            return 0;
        }

        if let Some(val) = store.get(&x) {
            return *val;
        }

        // compute value
        let mut best = 0;
        let step = x.step();
        if costs.can_ore(x) { // build ore robot
            best = best.max(f(costs, store, State {
                key: step.key + State::ORE_BOT - costs.b_ore as u64,
            }));
        }
        if costs.can_clay(x) { // build clay robot
            best = best.max(f(costs, store, State {
                key: step.key + State::CLAY_BOT - costs.c_ore as u64,
            }));
        }
        if costs.can_obs(x) { // build obsidian robot
            best = best.max(f(costs, store, State {
                key: step.key + State::OBS_BOT - costs.o_ore as u64 - costs.o_clay as u64,
            }));
        }
        if costs.can_geo(x) { // build geode robot
            best = best.max(f(costs, store, State {
                key: step.key - costs.g_ore as u64 - costs.g_obs as u64,
            }) + (x.remaining() - 1));
        }

        best = best.max(f(costs, store, x.step())); // do nothing

        store.insert(x, best);
        best
    }

    let costs = Costs::new(bp);
    let mut store = fnv::FnvHashMap::default();
    let out = f(&costs, &mut store, State {
        key: (t_max as u64 * State::TIMESTEP) | State::ORE_BOT
    });
    out
}

fn solve1(input: &Input) -> Result<usize> {
    Ok(input.par_iter().map(|bp| max_geodes(bp, 24) * bp.index).sum())
}

fn solve2(input: &Input) -> Result<usize> {
    Ok(input[..3].par_iter()
                 .map(|bp| max_geodes(bp, 32))
                 .product())
}

problem!(load_input => Vec<Blueprint> => (solve1, solve2));

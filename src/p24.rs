use anyhow::Result;

#[derive(Clone, Debug)]
/// Map of which blizzards are in which cells
///
/// Blizzards are represented as set bits in a bitmask, and one bitmap is stored per row and
/// column. Each blizzard direction is stored in its own vector, and the blizzard state of any
/// given cell on the board can be tested with simple bitwise operations.
struct Blizzards {
    // up/down blizzards. high-order bits store the bottom of the board.
    up: Vec<u128>,
    down: Vec<u128>,
    height: u32,

    // left/right blizzards. high-order bits store the right of the board.
    left: Vec<u128>,
    right: Vec<u128>,
    width: u32,
}

impl Blizzards {
    fn step(&mut self) {
        /// Rotate the `k` lowest-order bits of N leftwards
        fn rot_left(n: u128, k: u32) -> u128 {
            let mask = (1 << k) - 1;
            let b = (n >> (k-1)) & 1;
            ((n << 1) & mask) | b
        }

        /// Rotate the `k` lowest-order bits of N rightwards
        fn rot_right(n: u128, k: u32) -> u128 {
            let b = n & 1;
            (n >> 1) | (b << (k-1))
        }

        for x in self.up.iter_mut() {
            *x = rot_right(*x, self.height);
        }
        for x in self.down.iter_mut() {
            *x = rot_left(*x, self.height);
        }

        for x in self.left.iter_mut() {
            *x = rot_right(*x, self.width);
        }
        for x in self.right.iter_mut() {
            *x = rot_left(*x, self.width);
        }
    }

    fn is_free(&self, (x, y): (usize, usize)) -> bool {
        let ud = self.up[x] | self.down[x];
        let lr = self.left[y] | self.right[y];
        ((ud & (1 << y)) | (lr & (1 << x))) == 0
    }
}

struct Problem {
    /// Entry position above the first row
    enter_col: usize,

    /// Exit position below the last row
    leave_col: usize,

    /// Initial storm state
    storms: Blizzards,
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    #[derive(Copy, Clone, PartialEq, Eq)]
    enum Cell {
        Wall, Empty, Up, Down, Left, Right,
    }

    impl TryFrom<char> for Cell {
        type Error = anyhow::Error;

        fn try_from(x: char) -> Result<Self> {
            Ok(match x {
                '#' => Cell::Wall,
                '.' => Cell::Empty,
                '>' => Cell::Right,
                '<' => Cell::Left,
                '^' => Cell::Up,
                'v' => Cell::Down,
                c => anyhow::bail!("Invalid grid character: {}", c),
            })
        }
    }

    let cells = crate::util::load_grid::<Cell>(input)?;

    let main_width = cells.width() - 2; // ignore outer walls
    let main_height = cells.height() - 2;
    anyhow::ensure!(!(main_height > 128 || main_width > 128),
                    "Grid is too large for this implementation");
    let enter_col = cells.row_iter(0).skip(1).position(|c| matches!(c, Cell::Empty))
                   .ok_or_else(|| anyhow::anyhow!("No entry position"))?;
    let leave_col = cells.row_iter(cells.height()-1).skip(1).position(|c| matches!(c, Cell::Empty))
                   .ok_or_else(|| anyhow::anyhow!("No entry position"))?;

    // build blizzard bitmasks
    let mut storms = Blizzards {
        width: main_width as u32,
        height: main_height as u32,
        up: Vec::with_capacity(main_width),
        down: Vec::with_capacity(main_width),
        left: Vec::with_capacity(main_height),
        right: Vec::with_capacity(main_height),
    };
    for col in 0..main_width {
        let mut up = 0;
        let mut down = 0;
        for (i, cell) in cells.col_iter(1+col).skip(1).enumerate() {
            if *cell == Cell::Up {
                up |= 1 << i;
            }
            if *cell == Cell::Down {
                down |= 1 << i;
            }
        }
        storms.up.push(up);
        storms.down.push(down);
    }
    for row in 0..main_height {
        let mut left = 0;
        let mut right = 0;
        for (i, cell) in cells.row_iter(1+row).skip(1).enumerate() {
            if *cell == Cell::Left {
                left |= 1 << i;
            }
            if *cell == Cell::Right {
                right |= 1 << i;
            }
        }
        storms.left.push(left);
        storms.right.push(right);
    }

    for p in cells.points() {
        let (x,y) = p.coords();
        if x == 0 || x == (main_width + 1) || y == 0 || y == (main_height + 1) {
            continue;
        }
        let x = x - 1;
        let y = y - 1;

        assert_eq!(*p == Cell::Empty, storms.is_free((x, y)), "Error at {},{}", x, y);
    }

    Ok(Problem {enter_col, leave_col, storms})
}

/// Find the length of the shortest sequence of moves, if any, which will take an actor from the
/// entry to the exit safely.
fn shortest(
    mut storms: Blizzards,
    start: (usize, usize),
    end: (usize, usize),
) -> Option<(usize, Blizzards)> {
    #[derive(Debug)]
    struct State {
        t: usize,
        pos: (usize, usize),
    }

    let mut states = std::collections::VecDeque::new();
    
    // advance until there's an open space below the entryway

    let max_w = storms.width as usize - 1;
    let max_h = storms.height as usize - 1;

    // queued positions for t+1
    let mut seen = crate::grid::Grid::filled(storms.width as usize, storms.height as usize, false);

    let mut next_storm = storms.clone();
    next_storm.step();

    let mut storm_t = 1;
    loop {
        if !storms.is_free(start) {
            storms.step();
            next_storm.step();
            storm_t += 1;
            continue;
        }
        states.push_back(State { t: storm_t, pos: start });

        while let Some(State {pos, t}) = states.pop_front() {
            if t > storm_t {
                storms.step();
                next_storm.step();
                storm_t += 1;

                // add state for waiting
                if storms.is_free(start) && !*seen.get(start) {
                    states.push_back(State { pos: start, t: storm_t });
                }
                seen.fill(false);

            }
            if !storms.is_free(pos) {
                continue;
            }
            if pos == end {
                return Some((t, storms));
            }

            // try to move in every direction
            if pos.0 > 0 && next_storm.is_free((pos.0 - 1, pos.1)) && !*seen.get((pos.0 - 1, pos.1)) {
                states.push_back(State { pos: (pos.0 - 1, pos.1), t: t+1 });
                seen.set((pos.0 - 1, pos.1), true);
            }
            if pos.0 < max_w && next_storm.is_free((pos.0 + 1, pos.1)) && !*seen.get((pos.0 + 1, pos.1)) {
                states.push_back(State { pos: (pos.0 + 1, pos.1), t: t+1 });
                seen.set((pos.0 + 1, pos.1), true);
            }

            if pos.1 > 0 && next_storm.is_free((pos.0, pos.1 - 1)) && !*seen.get((pos.0, pos.1 - 1)) {
                states.push_back(State { pos: (pos.0, pos.1 - 1), t: t+1 });
                seen.set((pos.0, pos.1 - 1), true);
            }
            if pos.1 < max_h && next_storm.is_free((pos.0, pos.1 + 1)) && !*seen.get((pos.0, pos.1 + 1)) {
                states.push_back(State { pos: (pos.0, pos.1 + 1), t: t+1 });
                seen.set((pos.0, pos.1 + 1), true);
            }
            states.push_back(State { t: t+1, pos });
        }
    }
}

fn solve1(input: &Input) -> Result<usize> {
    let entry_pos = (input.enter_col, 0);
    let exit_pos = (input.leave_col, input.storms.height as usize - 1);
    shortest(input.storms.clone(), entry_pos, exit_pos)
        .map(|t| t.0)
        .ok_or_else(|| anyhow::anyhow!("Unable to find shortest path"))
}

fn solve2(input: &Input) -> Result<usize> {
    // journey there
    let entry_pos = (input.enter_col, 0);
    let exit_pos = (input.leave_col, input.storms.height as usize - 1);
    let (t0, mut storms) = shortest(input.storms.clone(), entry_pos, exit_pos)
                          .ok_or_else(|| anyhow::anyhow!("Unable to solve first leg"))?;
    storms.step();
    let (t1, mut storms) = shortest(storms, exit_pos, entry_pos)
                          .ok_or_else(|| anyhow::anyhow!("Unable to solve second leg"))?;
    storms.step();
    let (t2, _) = shortest(storms, entry_pos, exit_pos)
                 .ok_or_else(|| anyhow::anyhow!("Unable to solve third leg"))?;

    Ok(t0 + t1 + t2)
}

problem!(load_input => Problem => (solve1, solve2));

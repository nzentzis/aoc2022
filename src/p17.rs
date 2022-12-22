use anyhow::Result;

#[derive(Copy, Clone, Debug)]
enum Dir {
    Left, Right
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let mut s = String::new();
    input.read_to_string(&mut s)?;
    let mut lines = s.lines();
    let line = lines.next().ok_or_else(|| anyhow::anyhow!("Input is empty"))?;
    anyhow::ensure!(lines.next().is_none(), "Input has extra newlines");

    line.chars()
        .map(|c| match c {
            '>' => Ok(Dir::Right),
            '<' => Ok(Dir::Left),
            c   => Err(anyhow::anyhow!("Invalid char '{}' in input", c)),
        })
        .collect()
}

struct Shape {
    /// Representation of the lines of the shape, as bitmasks
    ///
    /// For example, the following shape would have the shown bitmask:
    ///
    /// ```text
    ///  ..#..          00000100 (0x04)
    ///  #.#.#          00010101 (0x15)
    ///  #####          00011111 (0x1f)
    ///  #.#.#          00010101 (0x15)
    /// ```
    ///
    /// The shape is always aligned with the bottom-right corner represented by the bitmasks.
    lines: [u8; 4],
    width: usize,
    height: usize,
}

const ROCKS: &[Shape] = &[
    Shape {lines: [ 0x00, 0x00, 0x00, 0x0f ], width: 4, height: 1 }, // - shape
    Shape {lines: [ 0x00, 0x02, 0x07, 0x02 ], width: 3, height: 3 }, // + shape
    Shape {lines: [ 0x00, 0x01, 0x01, 0x07 ], width: 3, height: 3 }, // reverse-L shape
    Shape {lines: [ 0x01, 0x01, 0x01, 0x01 ], width: 1, height: 4 }, // | shape
    Shape {lines: [ 0x00, 0x00, 0x03, 0x03 ], width: 2, height: 2 }, // # shape
];

#[derive(Clone)]
/// Board state representation
///
/// The origin of the coordinate system is at the bottom-right, with X coords going from 0 on the
/// right-hand side of the column to 7 on the left-hand side. Y coordinates start at 0 on the
/// bottom row and go up from there.
///
/// The position of a shape is considered to be the location of its lower-right corner.
struct Board {
    /// Rows of this board
    ///
    /// The last element of this vector represents the topmost non-empty row on the board
    rows: Vec<u8>,
}

impl Board {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
        }
    }

    fn drop_pos(&self, width: usize) -> (usize, usize) {
        (7 - (width + 2), // offset width, then 2 extra empty cells from the left edge
         self.rows.len() + 3)
    }

    /// Check whether the given shape at a specified location intersects any cells on the board
    fn collides(&self, rock: &Shape, pos: (usize, usize)) -> bool {
        // check collisions for any overlapping row
        let row_min = pos.1.min(self.rows.len()); // lower bound of row range
        let row_max = (pos.1 + rock.height).min(self.rows.len()); // upper bound of row range
        if row_min >= self.rows.len() {
            return false;
        }

        let collide_rows = &self.rows[row_min..row_max];
        let shape_rows = &rock.lines[4-collide_rows.len()..];
        assert_eq!(collide_rows.len(), shape_rows.len());

        for (b_line, r_line) in collide_rows.iter().zip(shape_rows.iter().rev()) {
            if (*b_line & (*r_line << pos.0)) != 0 {
                return true;
            }
        }

        false
    }

    /// Update the board to place the shape's cells at the given location
    fn place(&mut self, rock: &Shape, pos: (usize, usize)) {
        // add new lines if needed
        let max_y = pos.1 + rock.height;
        if max_y > self.rows.len() {
            self.rows.resize(max_y, 0);
        }

        // we're now guaranteed to have enough lines - update our board cells
        for (line, shape_line) in self.rows[pos.1..pos.1 + rock.height]
                                 .iter_mut()
                                 .zip(rock.lines.iter().rev()) {
            *line |= (*shape_line) << pos.0;
        }
    }
}

/// Simulate the given input for a provided number of rocks
///
/// Return the height of the board after simulation completes.
fn simulate_floyd(input: &Input, rocks: usize) -> Result<usize> {
    const CONTEXT: usize = 256;

    #[derive(Clone)]
    struct State {
        board: Board,
        t: usize,
        rock_idx: usize,
    }

    fn step(input: &Input, mut s: State) -> State {
        // drop the rock
        let rock = &ROCKS[s.rock_idx];
        s.rock_idx = (s.rock_idx + 1) % ROCKS.len();
        let mut pos = s.board.drop_pos(rock.width);
        loop {
            // try to push in a direction
            let cand_pos = match input[s.t] {
                Dir::Left => {
                    ((pos.0 + 1).min(7 - rock.width), pos.1)
                }
                Dir::Right => {
                    (pos.0.saturating_sub(1), pos.1)
                }
            };
            if !s.board.collides(rock, cand_pos) {
                pos = cand_pos;
            }
            s.t = (s.t + 1) % input.len();

            // try to fall one step
            if pos.1 == 0 {
                // hit the floor - rock lands
                break;
            }
            let cand_pos = (pos.0, pos.1 - 1);
            if s.board.collides(rock, cand_pos) {
                break;
            }
            pos = cand_pos;
        }

        // rock landed
        s.board.place(rock, pos);

        s
    }

    fn board_compare(a: &State, b: &State) -> bool {
        let a_ctx = a.board.rows.len().saturating_sub(CONTEXT);
        let b_ctx = b.board.rows.len().saturating_sub(CONTEXT);
        a.t == b.t &&
            a.rock_idx == b.rock_idx &&
            a.board.rows[a_ctx..] == b.board.rows[b_ctx..]
    }

    let x0 = State {
        board: Board::new(),
        t: 0,
        rock_idx: 0,
    };
    let mut tortoise = step(input, x0.clone());
    let mut hare = step(input, step(input, x0.clone()));
    while !board_compare(&tortoise, &hare) {
        tortoise = step(input, tortoise);
        hare = step(input, step(input, hare));
    }

    let mut offset = 0;
    tortoise = x0.clone();
    while !board_compare(&tortoise, &hare) {
        tortoise = step(input, tortoise);
        hare = step(input, hare);
        offset += 1;
    }

    let period_start_state = tortoise.clone();

    let mut period = 1;
    hare = step(input, tortoise.clone());
    while !board_compare(&tortoise, &hare) {
        hare = step(input, hare);
        period += 1;
    }
    std::mem::drop(tortoise);
    std::mem::drop(hare);

    // At this point, we know that the input board grows periodically with the computed period and
    // offset from the start. If we need to simulate N steps, we can divide overall height growth
    // into three segments: the initial sequence (i.e. the steps up to `offset`), the periodic
    // component (i.e. `floor(N / period)` times the height offset from the periodic component),
    // and the final offset (the height offset of `N % period` more steps after that point.
    //
    // Compute the final height by simulating the first and last part, short-circuiting the actual
    // periodic components.
    let mut x = x0;
    for _ in 0..rocks.min(offset) {
        x = step(input, x);
    }
    if rocks <= offset {
        return Ok(x.board.rows.len());
    }

    // now we're at the start of the loop - skip the periodic component
    let reps = (rocks - offset) / period;
    let rep_offset = (rocks - offset) % period;

    let rep_height = if reps > 0 {
        let start_h = x.board.rows.len(); // calculate height growth of periodic component
        let mut temp = period_start_state;
        for _ in 0..period {
            temp = step(input, temp);
        }
        let end_h = temp.board.rows.len();
        let delta = end_h - start_h;
        delta * reps
    } else {
        0
    };

    // also add offset, if needed
    for _ in 0..rep_offset {
        x = step(input, x);
    }

    Ok(x.board.rows.len() + rep_height)
}

fn solve1(input: &Input) -> Result<usize> {
    simulate_floyd(input, 2022)
}

fn solve2(input: &Input) -> Result<usize> {
    simulate_floyd(input, 1000000000000)
}

problem!(load_input => Vec<Dir> => (solve1, solve2));

use anyhow::{anyhow, Result};

#[derive(Copy, Clone)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Copy, Clone)]
struct Motion {
    dir: Direction,
    count: usize,
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    crate::util::read_lines(input, |line| {
        let parts = line.split_once(' ')
                        .ok_or_else(|| anyhow!("Missing separator on input line"))?;
        let dir = match parts.0 {
            "L" => Direction::Left,
            "R" => Direction::Right,
            "U" => Direction::Up,
            "D" => Direction::Down,
            _ => anyhow::bail!("Invalid direction character"),
        };
        let count = parts.1.parse::<usize>()?;

        Ok(Motion {dir, count})
    })
}

struct Rope {
    chain: Vec<(isize, isize)>,
}

impl Rope {
    fn new(pos: (isize, isize), length: usize) -> Self {
        assert!(length > 0);
        Self {
            chain: vec![pos; length]
        }
    }

    /// Move the head of the rope in the given direction
    fn move_head(&mut self, dir: Direction) {
        match dir {
            Direction::Left     => { self.chain[0].0 -= 1; }
            Direction::Right    => { self.chain[0].0 += 1; }
            Direction::Up       => { self.chain[0].1 -= 1; }
            Direction::Down     => { self.chain[0].1 += 1; }
        }

        self.adjust_tail();
    }

    /// Adjust the tail after moving the head
    fn adjust_tail(&mut self) {
        let mut head_ptr = 0;
        let mut tail_ptr = 1;
        while tail_ptr < self.chain.len() {
            let head = self.chain[head_ptr];
            let tail = &mut self.chain[tail_ptr];

            match (head.0 - tail.0, head.1 - tail.1) {
                // if not more than one away, do nothing
                (a, b) if a.abs().max(b.abs()) <= 1 => {
                }

                // cardinal directions
                (0, 2)  => { tail.1 += 1; }
                (0, -2) => { tail.1 -= 1; }
                (2, 0)  => { tail.0 += 1; }
                (-2, 0) => { tail.0 -= 1; }

                // diagonals
                (1, 2)  => { tail.0 += 1; tail.1 += 1; }
                (1, -2) => { tail.0 += 1; tail.1 -= 1; }
                (-1, 2) => { tail.0 -= 1; tail.1 += 1; }
                (-1, -2)=> { tail.0 -= 1; tail.1 -= 1; }

                (2, 1)  => { tail.0 += 1; tail.1 += 1; }
                (-2, 1) => { tail.0 -= 1; tail.1 += 1; }
                (2, -1) => { tail.0 += 1; tail.1 -= 1; }
                (-2, -1)=> { tail.0 -= 1; tail.1 -= 1; }

                (2, 2)  => { tail.0 += 1; tail.1 += 1; }
                (-2, 2) => { tail.0 -= 1; tail.1 += 1; }
                (2, -2) => { tail.0 += 1; tail.1 -= 1; }
                (-2, -2)=> { tail.0 -= 1; tail.1 -= 1; }

                (a, b) => unreachable!("{} {}", a, b)
            }

            head_ptr += 1;
            tail_ptr += 1;
        }
    }
}

fn solve1(input: &Input) -> Result<usize> {
    let mut rope = Rope::new((0, 0), 2);
    let mut positions = std::collections::HashSet::new();

    for motion in input.iter() {
        for _ in 0..motion.count {
            rope.move_head(motion.dir);
            positions.insert(rope.chain[1]);
        }
    }

    Ok(positions.len())
}

fn solve2(input: &Input) -> Result<usize> {
    let mut rope = Rope::new((0, 0), 10);
    let mut positions = std::collections::HashSet::new();

    for motion in input.iter() {
        for _ in 0..motion.count {
            rope.move_head(motion.dir);
            positions.insert(*rope.chain.last().unwrap());
        }
    }

    Ok(positions.len())
}

problem!(load_input => Vec<Motion> => (solve1, solve2));

use anyhow::{anyhow, Result};

use std::str::FromStr;

type Crate = char;

#[derive(Copy, Clone)]
struct Move {
    from: usize,
    to: usize,
    count: usize,
}

#[derive(Clone)]
struct Stacks {
    data: Vec<Vec<Crate>>,
}

impl Stacks {
    fn apply_move(&mut self, m: Move) -> Result<()> {
        assert!(m.to < self.data.len());
        assert!(m.from < self.data.len());

        for _ in 0..m.count {
            let item = self.data[m.from].pop().ok_or_else(|| anyhow!("Stack {} is empty", m.from))?;
            self.data[m.to].push(item);
        }

        Ok(())
    }

    fn apply_move_part2(&mut self, m: Move) -> Result<()> {
        assert!(m.to < self.data.len());
        assert!(m.from < self.data.len());
        anyhow::ensure!(self.data[m.from].len() >= m.count, "Insufficient crates in input stack");

        let start_idx = self.data[m.from].len() - m.count;
        let crates = self.data[m.from].drain(start_idx..).collect::<Vec<_>>();
        self.data[m.to].extend(crates);

        Ok(())
    }
}

struct Problem {
    stacks: Stacks,
    moves: Vec<Move>,
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    // parse initial state
    let stacks = {
        let mut content = String::new();
        while input.read_line(&mut content)? > 1 {
        }

        // look at the last line - how many columns are there?
        content.truncate(content.trim_end().len());
        let mut lines = content.lines().rev();

        let last_line = lines.next().ok_or_else(|| anyhow!("No trailing state line"))?;
        let column_positions = last_line.chars()
                              .enumerate()
                              .filter(|(_, c)| !c.is_whitespace())
                              .map(|(idx, _)| idx)
                              .collect::<Vec<_>>();

        // transpose into columns
        let mut stacks = vec![Vec::new(); column_positions.len()];
        for line in lines {
            let chars = line.chars().collect::<Vec<_>>();
            for (line_char, stack) in column_positions.iter()
                                     .map(|pos| chars[*pos])
                                     .zip(stacks.iter_mut())
                                     .filter(|(ch, _)| !ch.is_whitespace()) {
                stack.push(line_char);
            }
        }

        Stacks { data: stacks }
    };

    let moves = crate::util::read_lines_regex(input, r#"^move (\d+) from (\d+) to (\d+)$"#, |cap| {
        Ok(Move {
            count: usize::from_str(&cap[1])?,
            from: usize::from_str(&cap[2])? - 1, // offset indices
            to: usize::from_str(&cap[3])? - 1,
        })
    })?;

    Ok(Problem { stacks, moves })
}

fn solve1(input: &Input) -> Result<String> {
    let mut stacks = input.stacks.clone();

    for mv in &input.moves {
        stacks.apply_move(*mv)?;
    }

    Ok(stacks.data.into_iter().flat_map(|stack| stack.last().cloned()).collect())
}

fn solve2(input: &Input) -> Result<String> {
    let mut stacks = input.stacks.clone();

    for mv in &input.moves {
        stacks.apply_move_part2(*mv)?;
    }

    Ok(stacks.data.into_iter().flat_map(|stack| stack.last().cloned()).collect())
}

problem!(load_input => Problem => (solve1, solve2));

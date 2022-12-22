use anyhow::Result;

use std::str::FromStr;

use crate::grid::Grid;

pub enum Insn {
    Noop,
    AddX(i64),
}

impl FromStr for Insn {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "noop" {
            Ok(Self::Noop)
        } else if let Some(s) = s.strip_prefix("addx ") {
            Ok(Self::AddX(s.parse::<i64>()?))
        } else {
            Err(anyhow::anyhow!("Invalid instruction in input file"))
        }
    }
}

/// Simulate the machine, returning a history of the X register
///
/// The element at index `i` in the resulting array is the value of X after `i-1` cycles.
fn simulate(code: &[Insn]) -> Vec<i64> {
    let mut x = 1;
    let mut history = vec![1];

    for insn in code {
        match insn {
            Insn::Noop => {
                history.push(x);
            }
            Insn::AddX(n) => {
                history.push(x);
                x += n;
                history.push(x);
            }
        }
    }

    history
}

fn solve1(input: &Input) -> Result<i64> {
    let history = simulate(input);
    let strength = history.iter().enumerate()
                  .map(|(cycle, x)| (1+cycle as i64) * *x)
                  .collect::<Vec<_>>();

    Ok(strength[19] + strength[59] + strength[99] + strength[139] + strength[179] + strength[219])
}

fn solve2(input: &Input) -> Result<Grid<bool>> {
    let history = simulate(input);
    let mut crt = Grid::filled(40, 6, false);

    let mut cycle = 0;
    for y in 0..6usize {
        for x in 0..40usize {
            let sprite_x = history[cycle];
            if (sprite_x - x as i64).abs() <= 1 {
                crt.set((x, y), true);
            }
            cycle += 1;
        }
    }

    Ok(crt)
}

problem!(crate::util::load_lines => Vec<Insn> => (solve1, solve2));

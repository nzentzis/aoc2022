use anyhow::{anyhow, Result};

use std::str::FromStr;

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let mut out = Vec::new();
    let mut current = Vec::new();

    let mut line = String::new();
    while input.read_line(&mut line)? > 0 {
        if line.trim().is_empty() {
            out.push(std::mem::take(&mut current));
        } else {
            let n = u64::from_str(line.trim())?;
            current.push(n);
        }
        line.clear();
    }

    if !current.is_empty() {
        out.push(current);
    }

    Ok(out)
}

fn solve1(elves: &Input) -> Result<u64> {
    elves.iter()
         .map(|seq| seq.iter().cloned().sum::<u64>())
         .max()
         .ok_or_else(|| anyhow!("No elf data"))
}

fn solve2(elves: &Input) -> Result<u64> {
    let mut data = elves.iter()
                  .map(|seq| seq.iter().cloned().sum::<u64>())
                  .collect::<Vec<_>>();
    data.sort_unstable();

    Ok(data[data.len()-3..].iter().sum::<u64>())
}

problem!(load_input => Vec<Vec<u64>> => (solve1, solve2));

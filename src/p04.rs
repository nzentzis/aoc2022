use anyhow::{anyhow, Result};

use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
struct Range {
    start: u8,
    end: u8,
}

impl Range {
    fn contains(&self, other: &Self) -> bool {
        (self.start <= other.start) && (self.end >= other.end)
    }

    fn overlaps(&self, other: &Self) -> bool {
        // easier to express as 'not disjoint'
        !((self.end < other.start) || (self.start > other.end))
    }
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    fn parse_range(s: &str) -> Result<Range> {
        let (a, b) = s.split_once('-').ok_or_else(|| anyhow!("No separating dash on input"))?;
        let start = u8::from_str(a)?;
        let end = u8::from_str(b)?;

        Ok(Range {start, end})
    }

    crate::util::read_lines(input, |line| {
        let (a, b) = line.split_once(',')
                    .ok_or_else(|| anyhow!("No splitting comma on input line"))?;
        Ok((parse_range(a)?,
            parse_range(b)?))
    })
}

fn solve1(input: &Input) -> Result<usize> {
    Ok(input.iter().filter(|(a, b)| a.contains(b) || b.contains(a)).count())
}

fn solve2(input: &Input) -> Result<usize> {
    Ok(input.iter().filter(|(a, b)| a.overlaps(b)).count())
}

problem!(load_input => Vec<(Range, Range)> => (solve1, solve2));

use anyhow::{anyhow, Result};

use crate::util;

#[derive(Copy, Clone, PartialEq)]
enum Ending { Lose, Draw, Win }

const ENDINGS: &[Ending] = &[Ending::Lose, Ending::Draw, Ending::Win];

#[derive(Copy, Clone, PartialEq)]
enum Play {
    Rock,
    Paper,
    Scissors,
}

impl Play {
    fn score_for(&self) -> usize {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }

    fn match_score(&self, opponent: &Self) -> usize {
        match (self, opponent) {
            (x, y) if x == y => 3,

            (Play::Rock,        Play::Scissors) => 6,
            (Play::Paper,       Play::Rock) => 6,
            (Play::Scissors,    Play::Paper) => 6,
            _ => 0,
        }
    }

    /// Get a move we can play to produce the given ending, if the opponent will play this
    fn move_for(&self, result: Ending) -> Self {
        match (self, result) {
            (opp, Ending::Draw) => *opp,

            (Play::Rock, Ending::Lose) => Play::Scissors,
            (Play::Rock, Ending::Win) => Play::Paper,

            (Play::Paper, Ending::Lose) => Play::Rock,
            (Play::Paper, Ending::Win) => Play::Scissors,

            (Play::Scissors, Ending::Lose) => Play::Paper,
            (Play::Scissors, Ending::Win) => Play::Rock,
        }
    }
}

const MOVES: &[Play] = &[Play::Rock, Play::Paper, Play::Scissors];

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    util::read_lines(input, |line| {
        let a = line.chars().next().ok_or_else(|| anyhow!("No first value on line"))?;
        let b = line.chars().nth(2).ok_or_else(|| anyhow!("No first value on line"))?;

        Ok((a as u8 - b'A',
            b as u8 - b'X'))
    })
}

fn solve1(input: &Input) -> Result<usize> {
    Ok(input.iter()
            .map(|(them, me)| (MOVES[*me as usize], MOVES[*them as usize]))
            .map(|(me, them)| me.match_score(&them) + me.score_for())
            .sum())
}

fn solve2(input: &Input) -> Result<usize> {
    Ok(input.iter()
            .map(|(them, ending)| (MOVES[*them as usize], ENDINGS[*ending as usize]))
            .map(|(them, ending)| {
                let me = them.move_for(ending);
                me.match_score(&them) + me.score_for()
            })
            .sum())
}

problem!(load_input => Vec<(u8, u8)> => (solve1, solve2));

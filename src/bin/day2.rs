use anyhow::{bail, Context, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;

fn main() -> Result<()> {
    let rounds = parse_as_plys()?;

    let total_score: u32 = rounds.iter().map(Round::score).sum();
    dbg!(total_score);

    let total_score_2: u32 = parse_as_ply_and_outcome()?
        .iter()
        .map(|&(theirs, outcome)| Round::from_ply_and_outcome(theirs, outcome).score())
        .sum();
    dbg!(total_score_2);

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ply {
    Rock,
    Paper,
    Scissors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Win,
    Lose,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Round {
    theirs: Ply,
    ours: Ply,
}

impl Round {
    fn from_ply_and_outcome(theirs: Ply, outcome: Outcome) -> Round {
        let ours = match (theirs, outcome) {
            (Ply::Rock, Outcome::Win) => Ply::Paper,
            (Ply::Rock, Outcome::Lose) => Ply::Scissors,
            (Ply::Rock, Outcome::Draw) => Ply::Rock,
            (Ply::Paper, Outcome::Win) => Ply::Scissors,
            (Ply::Paper, Outcome::Lose) => Ply::Rock,
            (Ply::Paper, Outcome::Draw) => Ply::Paper,
            (Ply::Scissors, Outcome::Win) => Ply::Rock,
            (Ply::Scissors, Outcome::Lose) => Ply::Paper,
            (Ply::Scissors, Outcome::Draw) => Ply::Scissors,
        };

        Round { theirs, ours }
    }

    fn outcome(&self) -> Outcome {
        match (self.ours, self.theirs) {
            (Ply::Rock, Ply::Rock) => Outcome::Draw,
            (Ply::Rock, Ply::Paper) => Outcome::Lose,
            (Ply::Rock, Ply::Scissors) => Outcome::Win,
            (Ply::Paper, Ply::Rock) => Outcome::Win,
            (Ply::Paper, Ply::Paper) => Outcome::Draw,
            (Ply::Paper, Ply::Scissors) => Outcome::Lose,
            (Ply::Scissors, Ply::Rock) => Outcome::Lose,
            (Ply::Scissors, Ply::Paper) => Outcome::Win,
            (Ply::Scissors, Ply::Scissors) => Outcome::Draw,
        }
    }

    fn score(&self) -> u32 {
        let ours = match self.ours {
            Ply::Rock => 1,
            Ply::Paper => 2,
            Ply::Scissors => 3,
        };

        let outcome = match self.outcome() {
            Outcome::Win => 6,
            Outcome::Lose => 0,
            Outcome::Draw => 3,
        };

        ours + outcome
    }
}

fn parse_as_plys() -> Result<Vec<Round>> {
    input_lines()?
        .iter()
        .map(|line| {
            let (theirs, ours) = line
                .split_whitespace()
                .collect_tuple()
                .context("Too many elements on line")?;

            let theirs = match theirs {
                "A" => Ply::Rock,
                "B" => Ply::Paper,
                "C" => Ply::Scissors,
                _ => bail!("Unknown ply {}", theirs),
            };

            let ours = match ours {
                "X" => Ply::Rock,
                "Y" => Ply::Paper,
                "Z" => Ply::Scissors,
                _ => bail!("Unknown ply {}", ours),
            };

            Ok(Round { theirs, ours })
        })
        .collect()
}

fn parse_as_ply_and_outcome() -> Result<Vec<(Ply, Outcome)>> {
    input_lines()?
        .iter()
        .map(|line| {
            let (theirs, outcome) = line
                .split_whitespace()
                .collect_tuple()
                .context("Too many elements on line")?;

            let theirs = match theirs {
                "A" => Ply::Rock,
                "B" => Ply::Paper,
                "C" => Ply::Scissors,
                _ => bail!("Unknown ply {}", theirs),
            };

            let outcome = match outcome {
                "X" => Outcome::Lose,
                "Y" => Outcome::Draw,
                "Z" => Outcome::Win,
                _ => bail!("Unknown outcome {}", outcome),
            };

            Ok((theirs, outcome))
        })
        .collect()
}

use std::str::FromStr;

use anyhow::{Context, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;

fn main() -> Result<()> {
    let assignment_pairs = parse_input()?;

    let fully_containing_pairs = assignment_pairs
        .iter()
        .filter(|(first, second)| first.fully_contains(second) || second.fully_contains(first))
        .count();
    dbg!(fully_containing_pairs);

    let overlapping_pairs = assignment_pairs
        .iter()
        .filter(|(first, second)| first.overlaps(second))
        .count();
    dbg!(overlapping_pairs);

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Assignment {
    start: u32,
    end: u32,
}

impl Assignment {
    fn new(start: u32, end: u32) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    fn fully_contains(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end && self.end >= other.start
    }
}

impl FromStr for Assignment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = s.split('-').collect_tuple().context("Invalid assignment")?;
        Ok(Self::new(start.parse()?, end.parse()?))
    }
}

fn parse_input() -> Result<Vec<(Assignment, Assignment)>> {
    input_lines()?
        .into_iter()
        .map(|line| {
            let (first, second) = line
                .split(',')
                .collect_tuple()
                .context("More than two assignments on line")?;

            let first = first.parse()?;
            let second = second.parse()?;

            Ok((first, second))
        })
        .collect()
}

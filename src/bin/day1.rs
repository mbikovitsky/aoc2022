use anyhow::{Context, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;

fn main() -> Result<()> {
    let inventories = parse_input()?;

    let sorted_total_calories = inventories
        .iter()
        .map(|inventory| inventory.iter().sum::<u32>())
        .sorted()
        .collect_vec();

    let max = sorted_total_calories
        .last()
        .context("Expected at least one inventory")?;
    dbg!(max);

    assert!(sorted_total_calories.len() >= 3);
    let top_three = sorted_total_calories[sorted_total_calories.len() - 3..]
        .iter()
        .sum::<u32>();
    dbg!(top_three);

    Ok(())
}

fn parse_input() -> Result<Vec<Vec<u32>>> {
    input_lines()?
        .split(|line| line.is_empty())
        .map(|inventory| {
            inventory
                .iter()
                .map(|calories| Ok(calories.parse()?))
                .collect()
        })
        .collect()
}

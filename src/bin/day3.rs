use std::{borrow::Borrow, collections::HashSet};

use anyhow::{bail, Context, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;

fn main() -> Result<()> {
    let rucksacks = parse_input()?;

    dbg!(sum_common_item_priorities(&rucksacks)?);

    dbg!(sum_group_badge_priorities(&rucksacks)?);

    Ok(())
}

fn sum_common_item_priorities(rucksacks: &[Rucksack]) -> Result<u32> {
    let common_items: Result<Vec<char>> = rucksacks.iter().map(Rucksack::common_item).collect();
    let common_items = common_items?;

    let priorities: Result<Vec<u8>> = common_items.iter().copied().map(priority).collect();
    let priorities = priorities?;

    let priority_sum: u32 = priorities.iter().copied().map(u32::from).sum();

    Ok(priority_sum)
}

fn sum_group_badge_priorities(rucksacks: &[Rucksack]) -> Result<u32> {
    let priorities = find_group_badges(rucksacks, 3)?
        .map(priority)
        .collect::<Result<Vec<u8>>>()?;
    Ok(priorities.into_iter().map(u32::from).sum())
}

fn find_group_badges(
    rucksacks: &[Rucksack],
    group_size: usize,
) -> Result<impl Iterator<Item = char>> {
    if rucksacks.len() % group_size != 0 {
        bail!(
            "Number of rucksacks ({}) not divisible by group size ({})",
            rucksacks.len(),
            group_size
        );
    }

    let badges: Result<Vec<char>> = rucksacks
        .iter()
        .chunks(group_size)
        .into_iter()
        .map(|group| group.collect_tuple().unwrap())
        .map(|(a, b, c)| badge([a, b, c].into_iter()))
        .collect();
    let badges = badges?;

    Ok(badges.into_iter())
}

fn badge<I, R>(rucksacks: I) -> Result<char>
where
    I: IntoIterator<Item = R>,
    R: Borrow<Rucksack>,
{
    Ok(rucksacks
        .into_iter()
        .map(|rucksack| {
            rucksack
                .borrow()
                .first_compartment
                .union(&rucksack.borrow().second_compartment)
                .copied()
                .collect()
        })
        .reduce(|acc: HashSet<char>, elem| acc.intersection(&elem).copied().collect())
        .context("Expected at least one rucksack")?
        .into_iter()
        .exactly_one()?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rucksack {
    first_compartment: HashSet<char>,
    second_compartment: HashSet<char>,
}

impl Rucksack {
    fn common_item(&self) -> Result<char> {
        let item = self
            .first_compartment
            .intersection(&self.second_compartment)
            .copied()
            .exactly_one();
        if item.is_err() {
            bail!("More than one common item");
        }
        Ok(item.unwrap())
    }
}

fn priority(item: char) -> Result<u8> {
    match item {
        'a'..='z' => Ok(1 + item as u8 - b'a'),
        'A'..='Z' => Ok(27 + item as u8 - b'A'),
        _ => bail!("Invalid item {}", item),
    }
}

fn parse_input() -> Result<Vec<Rucksack>> {
    input_lines()?
        .iter()
        .map(|line| {
            let items = line.chars().collect_vec();

            if items.len() % 2 != 0 {
                bail!("Amount of items in rucksack is not even: {}", line);
            }

            let (first_compartment, second_compartment) = items.split_at(items.len() / 2);
            let first_compartment = first_compartment.iter().copied().collect();
            let second_compartment = second_compartment.iter().copied().collect();

            Ok(Rucksack {
                first_compartment,
                second_compartment,
            })
        })
        .collect()
}

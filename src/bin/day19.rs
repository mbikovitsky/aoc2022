use std::{
    collections::{HashMap, HashSet},
    ops::{Add, Sub},
};

use anyhow::{bail, Context, Result};
use aoc2022::util::input_lines;
use lazy_static::lazy_static;
use regex::Regex;

fn main() -> Result<()> {
    let blueprints = parse_input()?;

    let quality_levels: u32 = blueprints
        .iter()
        .enumerate()
        .map(|(index, blueprint)| {
            let id: u32 = index.try_into().unwrap();
            maximize_geodes(blueprint, 24) * (id + 1)
        })
        .sum();
    dbg!(quality_levels);

    Ok(())
}

fn maximize_geodes(blueprint: &Blueprint, total_time: u32) -> u32 {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct Node {
        resources: Resources,
        robots: Robots,
        time: u32,
    }

    let mut to_visit = vec![Node {
        resources: Resources::default(),
        robots: Robots {
            ore: 1,
            ..Default::default()
        },
        time: 0,
    }];

    let mut visited = HashSet::new();

    let neighbours = |node: Node| {
        let build = [
            // We can try building a robot ...
            if node.resources.is_enough_for(&blueprint.ore) {
                Some(Node {
                    resources: node.resources - blueprint.ore,
                    robots: node.robots
                        + Robots {
                            ore: 1,
                            ..Default::default()
                        },
                    time: node.time + 1,
                })
            } else {
                None
            },
            if node.resources.is_enough_for(&blueprint.clay) {
                Some(Node {
                    resources: node.resources - blueprint.clay,
                    robots: node.robots
                        + Robots {
                            clay: 1,
                            ..Default::default()
                        },
                    time: node.time + 1,
                })
            } else {
                None
            },
            if node.resources.is_enough_for(&blueprint.obsidian) {
                Some(Node {
                    resources: node.resources - blueprint.obsidian,
                    robots: node.robots
                        + Robots {
                            obsidian: 1,
                            ..Default::default()
                        },
                    time: node.time + 1,
                })
            } else {
                None
            },
            if node.resources.is_enough_for(&blueprint.geode) {
                Some(Node {
                    resources: node.resources - blueprint.geode,
                    robots: node.robots
                        + Robots {
                            geode: 1,
                            ..Default::default()
                        },
                    time: node.time + 1,
                })
            } else {
                None
            },
            // ... or not
            Some(Node {
                resources: node.resources,
                robots: node.robots,
                time: node.time + 1,
            }),
        ];

        // For each outcome, add resources from the already-existing robots
        build.into_iter().flatten().map(move |mut neighbour| {
            neighbour.resources.ore += node.robots.ore;
            neighbour.resources.clay += node.robots.clay;
            neighbour.resources.obsidian += node.robots.obsidian;
            neighbour.resources.geode += node.robots.geode;
            neighbour
        })
    };

    let mut max_geodes = 0;

    while let Some(current) = to_visit.pop() {
        if !visited.insert(current) {
            continue;
        }

        if current.time == total_time {
            max_geodes = max_geodes.max(current.resources.geode);
            continue;
        }

        for neighbour in neighbours(current) {
            if !visited.contains(&neighbour) {
                to_visit.push(neighbour);
            }
        }
    }

    max_geodes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Blueprint {
    ore: Price,
    clay: Price,
    obsidian: Price,
    geode: Price,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Price {
    ore: u32,
    clay: u32,
    obsidian: u32,
    geode: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Resources {
    ore: u32,
    clay: u32,
    obsidian: u32,
    geode: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Robots {
    ore: u32,
    clay: u32,
    obsidian: u32,
    geode: u32,
}

impl Add for Robots {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            ore: self.ore + rhs.ore,
            clay: self.clay + rhs.clay,
            obsidian: self.obsidian + rhs.obsidian,
            geode: self.geode + rhs.geode,
        }
    }
}

impl Sub<Price> for Resources {
    type Output = Self;

    fn sub(self, rhs: Price) -> Self::Output {
        Self {
            ore: self.ore - rhs.ore,
            clay: self.clay - rhs.clay,
            obsidian: self.obsidian - rhs.obsidian,
            geode: self.geode - rhs.geode,
        }
    }
}

impl Resources {
    fn is_enough_for(&self, price: &Price) -> bool {
        self.ore >= price.ore
            && self.clay >= price.clay
            && self.obsidian >= price.obsidian
            && self.geode >= price.geode
    }
}

fn parse_input() -> Result<Vec<Blueprint>> {
    input_lines()?.into_iter().map(|line| {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r#"Each (ore|clay|obsidian|geode) robot costs (\d+) (ore|clay|obsidian)(?: and (\d+) (ore|clay|obsidian)|)\."#).unwrap();
        }

        let mut robots = HashMap::new();

        for robot in REGEX.captures_iter(&line) {
            let kind = robot.get(1).unwrap().as_str();

            let cost1: u32 = robot.get(2).unwrap().as_str().parse()?;
            let material1 = robot.get(3).unwrap().as_str();

            let (cost2, material2) = if let Some(cost2) = robot.get(4) {
                let cost2:u32 = cost2.as_str().parse()?;
                let material2 = robot.get(5).unwrap().as_str();
                (cost2,material2)
            } else {
                (0, "")
            };

            if material1 == material2 {
                bail!("Expected two different material costs");
            }

            let mut price = Price::default();

            match material1 {
                "ore" => price.ore = cost1,
                "clay" => price.clay = cost1,
                "obsidian" => price.obsidian = cost1,
                _ => unreachable!()
            }

            match material2 {
                "ore" => price.ore = cost2,
                "clay" => price.clay = cost2,
                "obsidian" => price.obsidian = cost2,
                "" => {},
                _ => unreachable!()
            }

            robots.insert(kind.to_owned(), price);
        }

        let blueprint = Blueprint {
            ore: *robots.get("ore").context("Didn't find ore robot")?,
            clay: *robots.get("clay").context("Didn't find clay robot")?,
            obsidian: *robots.get("obsidian").context("Didn't find obsidian robot")?,
            geode: *robots.get("geode").context("Didn't find geode robot")?,
        };

        Ok(blueprint)
    }).collect()
}

use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result};
use aoc2022::util::input_lines;
use itertools::{iproduct, Itertools};
use lazy_static::lazy_static;
use petgraph::{
    algo::{bellman_ford, floyd_warshall},
    graph::DiGraph,
    graphmap::DiGraphMap,
};
use regex::Regex;

fn main() -> Result<()> {
    let valves = parse_input()?;

    let most_pressure = compute_most_pressure(&valves, 30);
    dbg!(most_pressure);

    Ok(())
}

fn compute_most_pressure(valves: &[Valve], end_time: u32) -> u32 {
    // For the bitmap
    assert!(valves.len() < 64);

    // First, build a map of the tunnel system
    let map = {
        // Directed graph because `floyd_warshall` doesn't like undirected ones...
        let mut map: DiGraphMap<&str, ()> = DiGraphMap::new();
        for valve in valves {
            for neighbour in &valve.tunnels {
                map.add_edge(&valve.name, neighbour, ());
            }
        }
        map
    };

    let shortest_paths = floyd_warshall(&map, |_| 1u32).unwrap();

    let working_valves_indices = valves
        .iter()
        .enumerate()
        .filter_map(|(index, valve)| {
            if valve.flow_rate != 0 {
                Some(index)
            } else {
                None
            }
        })
        .collect_vec();

    let turned_on_combos = working_valves_indices
        .iter()
        .powerset()
        .map(|indices| {
            indices
                .into_iter()
                .map(|index| 1u64 << index)
                .fold(0, |acc, val| acc | val)
        })
        .collect_vec();

    // Build a graph of possible states
    // State: (current time, current position, bitmap of turned on valves)
    // Weight: Increase in total pressure at end time if we take this edge, negated
    let (state_graph, start_state) = {
        let mut state_graph: DiGraph<(u32, usize, u64), f64> = DiGraph::new();

        let mut node_indices = HashMap::new();

        let start_state = (
            0,
            valves
                .iter()
                .find_position(|valve| valve.name == "AA")
                .unwrap()
                .0,
            0,
        );

        let all_states = || {
            std::iter::once(start_state).chain(iproduct!(
                0..end_time,
                working_valves_indices.iter().copied(),
                turned_on_combos.iter().copied()
            ))
        };

        for current_state in all_states() {
            node_indices.insert(current_state, state_graph.add_node(current_state));
        }

        for (time, position, turned_on_valves) in all_states() {
            let current_state = node_indices[&(time, position, turned_on_valves)];

            for &new_position in &working_valves_indices {
                // No sense in turning on valves that are already turned on
                if turned_on_valves & (1 << new_position) != 0 {
                    continue;
                }

                let distance = shortest_paths[&(
                    valves[position].name.as_str(),
                    valves[new_position].name.as_str(),
                )];

                // Filter out unreachable valves
                if distance == u32::MAX {
                    continue;
                }

                let new_time = time + distance + 1;
                if new_time >= end_time {
                    continue;
                }

                let new_state = (
                    new_time,
                    new_position,
                    turned_on_valves | (1 << new_position),
                );

                // How much pressure release can we get?
                let utility = (end_time - new_time) * valves[new_position].flow_rate;

                // Negate it, to employ shortest path search
                let utility: f64 = utility.into();
                let utility = -utility;

                let new_state = node_indices[&new_state];
                state_graph.add_edge(current_state, new_state, utility);
            }
        }

        (state_graph, node_indices[&start_state])
    };

    let paths = bellman_ford(&state_graph, start_state).unwrap();

    paths
        .distances
        .into_iter()
        .filter_map(|pressure| {
            let pressure = -pressure;

            assert!(!pressure.is_nan());

            if pressure.is_infinite() {
                return None;
            }

            assert!(0.0 <= pressure && pressure <= u32::MAX.into());

            let pressure: u32 = unsafe { pressure.to_int_unchecked() };
            Some(pressure)
        })
        .max()
        .unwrap()
}

#[derive(Debug, Clone)]
struct Valve {
    name: String,
    flow_rate: u32,
    tunnels: Vec<String>,
}

fn parse_input() -> Result<Vec<Valve>> {
    input_lines()?
        .into_iter()
        .map(|line| line.parse())
        .collect()
}

impl FromStr for Valve {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(
                r#"^Valve ([A-Z]{2}) has flow rate=(\d+); tunnels? leads? to valves? ([A-Z]{2}(?:, [A-Z]{2})*)$"#
            )
            .unwrap();
        }

        let captures = REGEX.captures(s).context("Invalid valve descriptor")?;

        let name = captures.get(1).unwrap().as_str().to_owned();

        let flow_rate = captures.get(2).unwrap().as_str().parse()?;

        let tunnels = captures
            .get(3)
            .unwrap()
            .as_str()
            .split(", ")
            .map(str::to_owned)
            .collect();

        Ok(Self {
            name,
            flow_rate,
            tunnels,
        })
    }
}

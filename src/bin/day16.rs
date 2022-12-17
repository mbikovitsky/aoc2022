use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use anyhow::{Context, Result};
use aoc2022::util::input_lines;
use itertools::{iproduct, Itertools};
use lazy_static::lazy_static;
use petgraph::{
    algo::floyd_warshall,
    graph::{DiGraph, NodeIndex},
    graphmap::DiGraphMap,
    visit::EdgeRef,
    Direction,
};
use regex::Regex;

fn main() -> Result<()> {
    let valves = parse_input()?;

    let most_pressure = compute_most_pressure(&valves, 30, "AA");
    dbg!(most_pressure);

    let most_pressure_two_agents = compute_most_pressure_two_agents(&valves, 26, "AA");
    dbg!(most_pressure_two_agents);

    Ok(())
}

fn compute_most_pressure(valves: &[Valve], total_time: u32, start_valve: &str) -> u32 {
    // First, build a map of the tunnel system
    let (map, start) = compress_valve_map(valves, start_valve);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct State {
        time: u32,
        position: NodeIndex,
        turned_on_valves: u64,
        utility: u32, // How much total pressure will be released when time runs out
    }

    let mut to_visit = vec![State {
        time: 0,
        position: start,
        turned_on_valves: 0,
        utility: 0,
    }];

    let mut discovered = HashSet::new();

    let mut best_utility = 0;

    while let Some(current) = to_visit.pop() {
        if !discovered.insert(current) {
            continue;
        }

        best_utility = best_utility.max(current.utility);

        // Find valves we can turn on
        for edge in map.edges_directed(current.position, Direction::Outgoing) {
            // Is this valve already turned on?
            if current.turned_on_valves & (1 << edge.target().index()) != 0 {
                continue;
            }

            // No sense in turning on valves that don't contribute anything
            let flow_rate = *map.node_weight(edge.target()).unwrap();
            if flow_rate == 0 {
                continue;
            }

            // New time is after moving to the new valve and turning it on
            let new_time = current.time + edge.weight() + 1;
            if new_time >= total_time {
                continue;
            }

            let new_state = State {
                time: new_time,
                position: edge.target(),
                turned_on_valves: current.turned_on_valves | (1 << edge.target().index()),
                utility: current.utility + (total_time - new_time) * flow_rate,
            };

            to_visit.push(new_state);
        }
    }

    best_utility
}

fn compute_most_pressure_two_agents(valves: &[Valve], total_time: u32, start_valve: &str) -> u32 {
    // First, build a map of the tunnel system
    let (map, start) = compress_valve_map(valves, start_valve);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct State {
        time1: u32,
        position1: NodeIndex,
        time2: u32,
        position2: NodeIndex,
        turned_on_valves: u64,
        utility: u32, // How much total pressure will be released when time runs out
    }

    let mut to_visit = vec![State {
        time1: 0,
        position1: start,
        time2: 0,
        position2: start,
        turned_on_valves: 0,
        utility: 0,
    }];

    let mut discovered = HashSet::new();

    let mut best_utility = 0;

    while let Some(current) = to_visit.pop() {
        if !discovered.insert(current) {
            continue;
        }

        best_utility = best_utility.max(current.utility);

        // Find valves the first agent can turn on
        for edge in map.edges_directed(current.position1, Direction::Outgoing) {
            // Is this valve already turned on?
            if current.turned_on_valves & (1 << edge.target().index()) != 0 {
                continue;
            }

            // No sense in turning on valves that don't contribute anything
            let flow_rate = *map.node_weight(edge.target()).unwrap();
            if flow_rate == 0 {
                continue;
            }

            // New time is after moving to the new valve and turning it on
            let new_time = current.time1 + edge.weight() + 1;
            if new_time >= total_time {
                continue;
            }

            let new_state = State {
                time1: new_time,
                position1: edge.target(),
                time2: current.time2,
                position2: current.position2,
                turned_on_valves: current.turned_on_valves | (1 << edge.target().index()),
                utility: current.utility + (total_time - new_time) * flow_rate,
            };

            to_visit.push(new_state);
        }

        // Find valves the second agent can turn on
        for edge in map.edges_directed(current.position2, Direction::Outgoing) {
            // Is this valve already turned on?
            if current.turned_on_valves & (1 << edge.target().index()) != 0 {
                continue;
            }

            // No sense in turning on valves that don't contribute anything
            let flow_rate = *map.node_weight(edge.target()).unwrap();
            if flow_rate == 0 {
                continue;
            }

            // New time is after moving to the new valve and turning it on
            let new_time = current.time2 + edge.weight() + 1;
            if new_time >= total_time {
                continue;
            }

            let new_state = State {
                time1: current.time1,
                position1: current.position1,
                time2: new_time,
                position2: edge.target(),
                turned_on_valves: current.turned_on_valves | (1 << edge.target().index()),
                utility: current.utility + (total_time - new_time) * flow_rate,
            };

            to_visit.push(new_state);
        }
    }

    best_utility
}

fn compress_valve_map(valves: &[Valve], start_valve: &str) -> (DiGraph<u32, u32>, NodeIndex) {
    // Directed graph because `floyd_warshall` doesn't like undirected ones...
    let mut map: DiGraphMap<&str, ()> = DiGraphMap::new();
    for valve in valves {
        for neighbour in &valve.tunnels {
            map.add_edge(&valve.name, neighbour, ());
        }
    }

    // We only care about the working valves, and the starting one,
    // so let's keep just them

    let mut compressed = DiGraph::new();

    let shortest_paths = floyd_warshall(&map, |_| 1u32).unwrap();

    let interesting_valves_indices = valves
        .iter()
        .enumerate()
        .filter_map(|(index, valve)| {
            if valve.name == start_valve || valve.flow_rate != 0 {
                Some(index)
            } else {
                None
            }
        })
        .collect_vec();

    let valve_index_to_node_index: HashMap<_, _> = interesting_valves_indices
        .iter()
        .map(|&valve_index| {
            let node_index = compressed.add_node(valves[valve_index].flow_rate);
            assert!(node_index.index() < 64); // For the bitmap
            (valve_index, node_index)
        })
        .collect();

    for (&a, &b) in iproduct!(&interesting_valves_indices, &interesting_valves_indices) {
        if a == b {
            continue;
        }

        let distance = shortest_paths[&(valves[a].name.as_str(), valves[b].name.as_str())];

        if distance == u32::MAX {
            continue;
        }

        compressed.add_edge(
            valve_index_to_node_index[&a],
            valve_index_to_node_index[&b],
            distance,
        );
    }

    let start = interesting_valves_indices
        .into_iter()
        .filter(|&index| valves[index].name == start_valve)
        .exactly_one()
        .expect("Expected to find exactly one start valve 'AA'");
    let start = valve_index_to_node_index[&start];

    (compressed, start)
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

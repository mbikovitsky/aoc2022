use std::collections::{HashMap, HashSet};

use anyhow::Result;
use aoc2022::util::input_lines;
use itertools::{Itertools, MinMaxResult};
use nalgebra::{Point2, Vector2};

fn main() -> Result<()> {
    let mut positions = parse_input();

    let empty_tiles = compute_empty_tiles(&mut positions.clone(), 10);
    dbg!(empty_tiles);

    let rounds_until_steady = simulate_until_steady_state(&mut positions);
    dbg!(rounds_until_steady + 1);

    Ok(())
}

fn compute_empty_tiles(positions: &mut HashSet<Point2<i32>>, rounds: usize) -> u32 {
    if positions.is_empty() {
        return 0;
    }

    // Simulate movement
    for round in 0..rounds {
        simulate1(positions, round);
    }

    // Compute bounding rectangle
    let (min_x, max_x) = match positions.iter().minmax_by_key(|point| point.x) {
        MinMaxResult::NoElements => unreachable!(),
        MinMaxResult::OneElement(minmax) => (minmax.x, minmax.x),
        MinMaxResult::MinMax(min, max) => (min.x, max.x),
    };
    let (min_y, max_y) = match positions.iter().minmax_by_key(|point| point.y) {
        MinMaxResult::NoElements => unreachable!(),
        MinMaxResult::OneElement(minmax) => (minmax.y, minmax.y),
        MinMaxResult::MinMax(min, max) => (min.y, max.y),
    };
    let width = min_x.abs_diff(max_x) + 1;
    let height = min_y.abs_diff(max_y) + 1;

    let total_tiles = width * height;
    let occupied_tiles: u32 = positions.len().try_into().unwrap();
    total_tiles - occupied_tiles
}

fn simulate_until_steady_state(positions: &mut HashSet<Point2<i32>>) -> usize {
    for round in 0.. {
        let previous = positions.clone();
        simulate1(positions, round);
        if &previous == positions {
            return round;
        }
    }
    unreachable!()
}

fn simulate1(positions: &mut HashSet<Point2<i32>>, round: usize) {
    const PROPOSAL_ORDER: [([Vector2<i32>; 3], Vector2<i32>); 4] = [
        ([N, NE, NW], N),
        ([S, SE, SW], S),
        ([W, NW, SW], W),
        ([E, NE, SE], E),
    ];

    // Maps a point to move to, to the elves that want to move there
    let mut proposals: HashMap<Point2<i32>, Vec<Point2<i32>>> = HashMap::new();

    // First half of the round
    for position in &*positions {
        // If no other Elves are in one of those eight positions,
        // the Elf does not do anything during this round.
        if !adjacent_points(position)
            .into_iter()
            .any(|neighbour| positions.contains(&neighbour))
        {
            continue;
        }

        // Otherwise, the Elf looks in each of four directions in the following
        // order and proposes moving one step in the first valid direction.
        for i in round..round + PROPOSAL_ORDER.len() {
            let i = i % PROPOSAL_ORDER.len();
            let (checks, direction) = PROPOSAL_ORDER[i];
            if !checks
                .into_iter()
                .any(|check| positions.contains(&(position + check)))
            {
                proposals
                    .entry(position + direction)
                    .or_default()
                    .push(*position);
                break;
            }
        }
    }

    for (proposal, contenders) in proposals {
        assert!(!contenders.is_empty());
        if contenders.len() > 1 {
            continue;
        }
        positions.remove(&contenders[0]);
        positions.insert(proposal);
    }
}

const N: Vector2<i32> = Vector2::new(0, -1);
const NE: Vector2<i32> = Vector2::new(1, -1);
const E: Vector2<i32> = Vector2::new(1, 0);
const SE: Vector2<i32> = Vector2::new(1, 1);
const S: Vector2<i32> = Vector2::new(0, 1);
const SW: Vector2<i32> = Vector2::new(-1, 1);
const W: Vector2<i32> = Vector2::new(-1, 0);
const NW: Vector2<i32> = Vector2::new(-1, -1);

fn adjacent_points(point: &Point2<i32>) -> [Point2<i32>; 8] {
    [
        point + N,
        point + NE,
        point + E,
        point + SE,
        point + S,
        point + SW,
        point + W,
        point + NW,
    ]
}

fn parse_input() -> HashSet<Point2<i32>> {
    input_lines()
        .unwrap()
        .into_iter()
        .enumerate()
        .flat_map(|(y, line)| {
            let y: i32 = y.try_into().unwrap();
            line.char_indices()
                .filter_map(|(x, c)| {
                    if c == '#' {
                        let x: i32 = x.try_into().unwrap();
                        Some(Point2::new(x, y))
                    } else {
                        None
                    }
                })
                .collect_vec()
        })
        .collect()
}

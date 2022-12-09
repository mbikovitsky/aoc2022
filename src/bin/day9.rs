use std::{num::NonZeroU32, str::FromStr};

use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;
use nalgebra::{Point2, Vector2};

fn main() -> Result<()> {
    let movements = parse_input();

    let unique_locations1 = simulate1(movements.iter().copied())
        .into_iter()
        .unique()
        .count();
    dbg!(unique_locations1);

    let unique_locations10 = simulate_many(movements.iter().copied(), NonZeroU32::new(9).unwrap())
        .into_iter()
        .unique()
        .count();
    dbg!(unique_locations10);

    Ok(())
}

fn simulate_many(
    movements: impl IntoIterator<Item = Vector2<i32>>,
    knots: NonZeroU32,
) -> Vec<Point2<i32>> {
    let mut knot_positions = simulate1(movements);

    for _ in 0..knots.get() - 1 {
        // Convert positions to movements

        let mut movements = vec![];

        for i in 1..knot_positions.len() {
            let delta = knot_positions[i] - knot_positions[i - 1];

            assert!(delta.x.abs() <= 1);
            assert!(delta.y.abs() <= 1);

            movements.push(delta);
        }

        // Move the next knot

        knot_positions = simulate1(movements);
    }

    knot_positions
}

fn simulate1(movements: impl IntoIterator<Item = Vector2<i32>>) -> Vec<Point2<i32>> {
    let mut tail_visited = vec![Point2::new(0, 0)];
    let mut head_position = Point2::<i32>::new(0, 0);

    for movement in movements {
        // Move the head

        head_position += movement;

        // Move the tail

        let tail_position = *tail_visited.last().unwrap();

        let delta = head_position - tail_position;
        assert!(delta.x.abs() <= 2);
        assert!(delta.y.abs() <= 2);

        if delta.x.abs() == 2 || delta.y.abs() == 2 {
            tail_visited.push(tail_position + Vector2::new(delta.x.signum(), delta.y.signum()));
        }
    }

    tail_visited
}

fn parse_input() -> Vec<Vector2<i32>> {
    input_lines()
        .expect("Couldn't parse input")
        .into_iter()
        .flat_map(|line| {
            let (direction, steps) = line
                .split_whitespace()
                .collect_tuple()
                .expect("Invalid string format");

            let direction = direction.parse().expect("Invalid direction");

            (0u8..steps.parse().expect("Invalid number of steps")).map(move |_| match direction {
                Direction::Up => Vector2::new(0, 1),
                Direction::Down => Vector2::new(0, -1),
                Direction::Left => Vector2::new(-1, 0),
                Direction::Right => Vector2::new(1, 0),
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Self::Up),
            "D" => Ok(Self::Down),
            "L" => Ok(Self::Left),
            "R" => Ok(Self::Right),
            _ => bail!("Invalid direction {}", s),
        }
    }
}

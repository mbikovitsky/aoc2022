use std::fmt::Display;

use anyhow::Result;
use aoc2022::util::input_lines;
use itertools::Itertools;
use nalgebra::Point2;
use ndarray::{Array2, ArrayViewMut2, Axis};
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map_res,
    multi::separated_list1,
    sequence::tuple,
    IResult,
};

const SAND_SOURCE: Point2<usize> = Point2::new(500, 0);

fn main() -> Result<()> {
    let mut map = parse_input();

    let grains = simulate_until_fall_off(map.clone().view_mut());
    dbg!(grains);

    let new_height = map.shape()[0] + 2;

    // In the worst case, with no obstructions, sand will fall from the source
    // and form a pyramid with a base of this width (1, 3, 5, ...)
    let pyramid_base = 1 + 2 * (new_height - 2);

    // To avoid shifting our map to the right, assume that half of the pyramid
    // base fits in the existing map.
    assert!(pyramid_base / 2 <= SAND_SOURCE.x);

    // Now extend the map...

    // Horizontally:
    // - SAND_SOURCE.x + pyramid_base / 2 is the right-most coordinate in the worst case
    // - +1 to pad it on the right with a rock floor
    // - +1 to get the actual width
    if map.shape()[1] < SAND_SOURCE.x + pyramid_base / 2 + 2 {
        map.append(
            Axis(1),
            Array2::from_shape_simple_fn(
                (
                    map.shape()[0],
                    (SAND_SOURCE.x + pyramid_base / 2 + 2) - map.shape()[1],
                ),
                || Tile::Air,
            )
            .view(),
        )
        .unwrap();
    }

    // Vertically:
    map.append(
        Axis(0),
        Array2::from_shape_simple_fn((1, map.shape()[1]), || Tile::Air).view(),
    )
    .unwrap();
    map.append(
        Axis(0),
        Array2::from_shape_simple_fn((1, map.shape()[1]), || Tile::Rock).view(),
    )
    .unwrap();

    let grains = simulate_until_source_blocked(map.view_mut());
    dbg!(grains);

    Ok(())
}

fn simulate_until_source_blocked(mut map: ArrayViewMut2<Tile>) -> usize {
    let mut grains = 0;
    loop {
        match simulate_grain_until_rest(map.view_mut(), SAND_SOURCE) {
            Some(new_position) => {
                grains += 1;
                if new_position == SAND_SOURCE {
                    return grains;
                }
            }
            None => {
                panic!("Grain {} fell off the map", grains);
            }
        }
    }
}

/// Simulates the fall of sand until the first grain falls off the map.
/// Returns the number of grains simulated.
fn simulate_until_fall_off(mut map: ArrayViewMut2<Tile>) -> usize {
    let mut grains = 0;
    loop {
        if simulate_grain_until_rest(map.view_mut(), SAND_SOURCE).is_none() {
            return grains;
        }
        grains += 1;
    }
}

/// Simulates the fall of a single grain of sand, until it comes to rest.
/// Returns the final resting place, or `None` if the grain went off the map.
fn simulate_grain_until_rest(
    mut map: ArrayViewMut2<Tile>,
    mut grain: Point2<usize>,
) -> Option<Point2<usize>> {
    assert_eq!(map[(grain.y, grain.x)], Tile::Air);
    map[(grain.y, grain.x)] = Tile::Sand;

    loop {
        let new_position = simulate_grain_one_step(map.view_mut(), grain);
        match new_position {
            None => return None,
            Some(new_position) => {
                if new_position == grain {
                    return Some(grain);
                } else {
                    grain = new_position;
                }
            }
        }
    }
}

/// Simulates a single fall step of a grain of sand.
/// Returns the grain's new position, or `None` if it would go off the map.
fn simulate_grain_one_step(
    mut map: ArrayViewMut2<Tile>,
    grain: Point2<usize>,
) -> Option<Point2<usize>> {
    assert_eq!(map[(grain.y, grain.x)], Tile::Sand);

    let new_position = if grain.y == map.shape()[0] - 1 {
        // Falling off straight down
        return None;
    } else if map[(grain.y + 1, grain.x)] == Tile::Air {
        Point2::new(grain.x, grain.y + 1)
    } else if grain.x == 0 {
        // Falling off to the left
        return None;
    } else if map[(grain.y + 1, grain.x - 1)] == Tile::Air {
        Point2::new(grain.x - 1, grain.y + 1)
    } else if grain.x == map.shape()[1] - 1 {
        // Falling off to the right
        return None;
    } else if map[(grain.y + 1, grain.x + 1)] == Tile::Air {
        Point2::new(grain.x + 1, grain.y + 1)
    } else {
        grain
    };

    map[(grain.y, grain.x)] = Tile::Air;
    map[(new_position.y, new_position.x)] = Tile::Sand;

    Some(new_position)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Sand,
    Rock,
    Air,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Sand => write!(f, "o"),
            Tile::Rock => write!(f, "#"),
            Tile::Air => write!(f, "."),
        }
    }
}

fn parse_input() -> Array2<Tile> {
    let paths = input_lines()
        .unwrap()
        .into_iter()
        .map(|line| {
            let (remainder, points) = parse_point_sequence(&line).unwrap();
            assert!(remainder.is_empty());
            points
        })
        .collect_vec();

    let max_x = paths.iter().flatten().map(|point| point.x).max().unwrap();
    let max_y = paths.iter().flatten().map(|point| point.y).max().unwrap();

    let mut result = Array2::from_shape_simple_fn((max_y + 1, max_x + 1), || Tile::Air);

    for path in paths {
        for pair in path.windows(2) {
            let (a, b) = pair.iter().collect_tuple().unwrap();

            if a.x == b.x {
                let x = a.x;

                let y_range = if a.y <= b.y { a.y..=b.y } else { b.y..=a.y };

                for y in y_range {
                    result[(y, x)] = Tile::Rock;
                }
            } else if a.y == b.y {
                let y = a.y;

                let x_range = if a.x <= b.x { a.x..=b.x } else { b.x..=a.x };

                for x in x_range {
                    result[(y, x)] = Tile::Rock;
                }
            } else {
                panic!("Path segment must be either horizontal or vertical");
            }
        }
    }

    result
}

fn parse_point_sequence(input: &str) -> IResult<&str, Vec<Point2<usize>>> {
    separated_list1(tuple((multispace0, tag("->"), multispace0)), parse_point)(input)
}

fn parse_point(input: &str) -> IResult<&str, Point2<usize>> {
    let mut coordinate = map_res(digit1, str::parse);

    let (input, x) = coordinate(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, y) = coordinate(input)?;

    Ok((input, Point2::new(x, y)))
}

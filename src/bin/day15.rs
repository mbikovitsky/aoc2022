use std::{
    collections::HashSet,
    ops::{Add, Range, RangeInclusive, Sub},
};

use aoc2022::util::input_lines;
use itertools::Itertools;
use lazy_static::lazy_static;
use nalgebra::{Point2, Scalar};
use regex::Regex;

fn main() -> anyhow::Result<()> {
    let data = parse_input();

    // Part 1

    const TARGET_Y: i64 = 2000000;

    let positions_without_beacon: i64 = find_positions_without_beacon(&data, TARGET_Y)
        .into_iter()
        .map(|range| {
            if range.is_empty() {
                0
            } else {
                range.end - range.start
            }
        })
        .sum();
    dbg!(positions_without_beacon);

    // Part 2

    const MIN_COORD: i64 = 0;
    const MAX_COORD: i64 = 4000000;

    let mut possible_locations = HashSet::new();
    for y in MIN_COORD..=MAX_COORD {
        let without_beacon = find_positions_without_beacon(&data, y);
        let possible_ranges = complement_ranges(&without_beacon);
        for range in possible_ranges {
            for x in range_intersection(range, MIN_COORD..=MAX_COORD) {
                possible_locations.insert(Point2::new(x, y));
            }
        }
    }

    for (_, beacon) in data {
        possible_locations.remove(&beacon);
    }

    let distress_beacon = possible_locations
        .into_iter()
        .exactly_one()
        .expect("More than one location for distress beacon found");
    let tuning_frequency = distress_beacon.x * 4000000 + distress_beacon.y;
    dbg!(tuning_frequency);

    Ok(())
}

fn find_positions_without_beacon(
    data: &[(Point2<i64>, Point2<i64>)],
    target_y: i64,
) -> Vec<Range<i64>> {
    // Find the range of X coordinates that each sensor says
    // cannot contain a beacon
    let mut ranges = data
        .iter()
        .filter_map(|(sensor, beacon)| {
            let radius = manhattan_distance(sensor, beacon);

            let vertical_distance = manhattan_distance(sensor, &Point2::new(sensor.x, target_y));

            if vertical_distance <= radius {
                let max_horizontal_distance = radius - vertical_distance;

                let mut min_x = sensor.x - max_horizontal_distance;
                let mut max_x = sensor.x + max_horizontal_distance;
                if beacon.y == target_y {
                    if beacon.x == min_x {
                        min_x += 1;
                    }
                    if beacon.x == max_x {
                        max_x -= 1;
                    }
                }

                let range = min_x..max_x + 1;
                assert!(!(beacon.y == target_y && range.contains(&beacon.x)));

                Some(range)
            } else {
                None
            }
        })
        .sorted_by_key(|range| range.start)
        .collect_vec();

    // Merge overlapping ranges
    let mut i = 0;
    while i < ranges.len() - 1 {
        if ranges[i].contains(&ranges[i + 1].start) {
            ranges[i] = merge_ranges(ranges[i].clone(), ranges[i + 1].clone());
            ranges.remove(i + 1);
        } else {
            i += 1;
        }
    }

    ranges
}

/// Merges two overlapping ranges.
/// Panics if the ranges do not overlap.
fn merge_ranges(first: Range<i64>, second: Range<i64>) -> Range<i64> {
    if first.start > second.start {
        return merge_ranges(second, first);
    }

    assert!(first.contains(&second.start));

    if second.end >= first.end {
        first.start..second.end
    } else {
        first
    }
}

/// Given a sequence of non-overlapping ranges, returns a sequence
/// of ranges that cover the spaces "in-between".
///
/// The results are unspecified if any of the ranges overlap.
fn complement_ranges(ranges: &[Range<i64>]) -> Vec<RangeInclusive<i64>> {
    if ranges.is_empty() {
        vec![i64::MIN..=i64::MAX]
    } else {
        let ranges = ranges
            .iter()
            .sorted_by_key(|range| range.start)
            .collect_vec();

        ranges
            .windows(2)
            .map(|pair| {
                assert_eq!(pair.len(), 2);
                assert!(!pair[0].contains(&pair[1].start));
                pair[0].end..=pair[1].start - 1
            })
            .chain([
                i64::MIN..=ranges.first().unwrap().start - 1,
                ranges.last().unwrap().end..=i64::MAX,
            ])
            .collect()
    }
}

fn range_intersection(
    first: RangeInclusive<i64>,
    second: RangeInclusive<i64>,
) -> RangeInclusive<i64> {
    *first.start().max(second.start())..=*first.end().min(second.end())
}

fn manhattan_distance<T>(a: &Point2<T>, b: &Point2<T>) -> T
where
    T: Scalar + Copy + Ord + Sub<Output = T> + Add<Output = T>,
{
    let delta_x = if a.x >= b.x { a.x - b.x } else { b.x - a.x };
    let delta_y = if a.y >= b.y { a.y - b.y } else { b.y - a.y };
    delta_x + delta_y
}

fn parse_input() -> Vec<(Point2<i64>, Point2<i64>)> {
    input_lines()
        .unwrap()
        .into_iter()
        .map(|line| {
            lazy_static! {
                static ref SENSOR_REGEX: Regex = Regex::new(
                    r#"^Sensor at x=(-?\d+), y=(-?\d+): closest beacon is at x=(-?\d+), y=(-?\d+)$"#
                )
                .unwrap();
            }

            let captures = SENSOR_REGEX.captures(&line).unwrap();

            let sensor_x = captures.get(1).unwrap().as_str().parse().unwrap();
            let sensor_y = captures.get(2).unwrap().as_str().parse().unwrap();

            let beacon_x = captures.get(3).unwrap().as_str().parse().unwrap();
            let beacon_y = captures.get(4).unwrap().as_str().parse().unwrap();

            (
                Point2::new(sensor_x, sensor_y),
                Point2::new(beacon_x, beacon_y),
            )
        })
        .collect()
}

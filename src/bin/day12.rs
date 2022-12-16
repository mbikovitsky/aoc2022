use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use anyhow::Result;
use aoc2022::util::{input_lines, manhattan_distance};
use itertools::Itertools;
use nalgebra::Point2;
use ndarray::{Array1, Array2, Axis};

fn main() -> Result<()> {
    let map = parse_input();

    dbg!(map.find_shortest_path(map.start).unwrap());

    let shortest_shortest_path = map
        .map
        .indexed_iter()
        .filter_map(|((y, x), &height)| {
            if height == b'a' {
                map.find_shortest_path(Point2::new(x, y))
            } else {
                None
            }
        })
        .min()
        .unwrap();
    dbg!(shortest_shortest_path);

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Map {
    map: Array2<u8>,
    start: Point2<usize>,
    end: Point2<usize>,
}

impl Map {
    fn find_shortest_path(&self, start: Point2<usize>) -> Option<u64> {
        // https://en.wikipedia.org/wiki/A*_search_algorithm
        
        struct Node {
            point: Point2<usize>,
            cost: u64,
        }

        impl PartialEq for Node {
            fn eq(&self, other: &Self) -> bool {
                self.cost == other.cost
            }
        }

        impl Eq for Node {}

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.cost.partial_cmp(&other.cost)
            }
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

        let heuristic = |point: Point2<usize>| -> u64 {
            manhattan_distance(&point, &self.end).try_into().unwrap()
        };

        let neighbours = |point: Point2<usize>| {
            let up = if point.y > 0 {
                Some(Point2::new(point.x, point.y - 1))
            } else {
                None
            };

            let down = if point.y < self.map.shape()[0] - 1 {
                Some(Point2::new(point.x, point.y + 1))
            } else {
                None
            };

            let left = if point.x > 0 {
                Some(Point2::new(point.x - 1, point.y))
            } else {
                None
            };

            let right = if point.x < self.map.shape()[1] - 1 {
                Some(Point2::new(point.x + 1, point.y))
            } else {
                None
            };

            up.into_iter()
                .chain(down)
                .chain(left)
                .chain(right)
                .filter(move |&destination| self.height_difference(point, destination) >= -1)
        };

        let mut open_set = BinaryHeap::new();
        open_set.push(Reverse(Node {
            point: start,
            cost: heuristic(start),
        }));

        let mut g_score = HashMap::new();
        g_score.insert(start, 0);

        let mut f_score = HashMap::new();
        f_score.insert(start, heuristic(start));

        while let Some(Reverse(current)) = open_set.pop() {
            if current.point == self.end {
                return Some(current.cost);
            }

            for neighbour in neighbours(current.point) {
                let tentative_g_score = g_score[&current.point] + 1;
                if tentative_g_score < *g_score.get(&neighbour).unwrap_or(&u64::MAX) {
                    g_score.insert(neighbour, tentative_g_score);
                    f_score.insert(neighbour, tentative_g_score + heuristic(neighbour));
                    open_set.push(Reverse(Node {
                        point: neighbour,
                        cost: f_score[&neighbour],
                    }));
                }
            }
        }

        None
    }

    fn height_difference(&self, a: Point2<usize>, b: Point2<usize>) -> i16 {
        let height_a: i16 = self.map[(a.y, a.x)].into();
        let height_b: i16 = self.map[(b.y, b.x)].into();
        height_a - height_b
    }
}

fn parse_input() -> Map {
    let rows = input_lines()
        .unwrap()
        .into_iter()
        .map(|line| {
            assert!(line.is_ascii());
            Array1::from_iter(line.bytes())
        })
        .collect_vec();
    let rows = rows.iter().map(|row| row.view()).collect_vec();

    let mut map = ndarray::stack(Axis(0), &rows).unwrap();

    let mut start = None;
    let mut end = None;

    for ((row, column), element) in map.indexed_iter_mut() {
        match element {
            b'S' => {
                assert!(start.is_none());
                start = Some(Point2::new(column, row));
                *element = b'a';
            }
            b'E' => {
                assert!(end.is_none());
                end = Some(Point2::new(column, row));
                *element = b'z';
            }
            b'a'..=b'z' => {}
            _ => panic!("Invalid character {}", element),
        }
    }

    let start = start.expect("Expected a start point");
    let end = end.expect("Expected an end point");

    Map { map, start, end }
}

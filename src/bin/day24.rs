use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use anyhow::{bail, Context, Result};
use aoc2022::util::{input_lines, manhattan_distance};
use nalgebra::{Point2, Vector2};

fn main() -> Result<()> {
    let map = parse_input()?;

    let shortest_path = map.find_shortest_path().context("Didn't find solution")?;
    dbg!(shortest_path);

    Ok(())
}

#[derive(Debug, Clone)]
struct Map {
    blizzards: Vec<Blizzard>,
    width: u8,
    height: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Blizzard {
    initial: Point2<u8>,
    direction: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Map {
    fn find_shortest_path(&self) -> Option<u32> {
        // https://en.wikipedia.org/wiki/A*_search_algorithm

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct Node {
            position: Option<Point2<u8>>,
            time: u32,
        }

        #[derive(Debug, Clone, Copy)]
        struct WeightedNode {
            data: Node,
            cost: u32,
        }

        impl PartialEq for WeightedNode {
            fn eq(&self, other: &Self) -> bool {
                self.cost == other.cost
            }
        }

        impl Eq for WeightedNode {}

        impl PartialOrd for WeightedNode {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.cost.partial_cmp(&other.cost)
            }
        }

        impl Ord for WeightedNode {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

        let end = Point2::new(self.width - 1, self.height - 1);
        let heuristic = |point: Option<Point2<u8>>| -> u32 {
            if let Some(point) = point {
                manhattan_distance(&point, &end).into()
            } else {
                // The start point is one above (0,0)
                let distance: u32 = manhattan_distance(&Point2::new(0, 0), &end).into();
                distance + 1
            }
        };

        let neighbours = |node: Node| {
            let up = if let Some(position) = node.position {
                if position.y > 0 {
                    Some(Some(Point2::new(position.x, position.y - 1)))
                } else {
                    None
                }
            } else {
                None
            };

            let down = if let Some(position) = node.position {
                if position.y < self.height - 1 {
                    Some(Some(Point2::new(position.x, position.y + 1)))
                } else {
                    None
                }
            } else {
                Some(Some(Point2::new(0, 0)))
            };

            let left = if let Some(position) = node.position {
                if position.x > 0 {
                    Some(Some(Point2::new(position.x - 1, position.y)))
                } else {
                    None
                }
            } else {
                None
            };

            let right = if let Some(position) = node.position {
                if position.x < self.width - 1 {
                    Some(Some(Point2::new(position.x + 1, position.y)))
                } else {
                    None
                }
            } else {
                None
            };

            let wait = Some(node.position);

            // Return only those positions where a blizzard won't be present
            // at the next time step
            [up, down, left, right, wait]
                .into_iter()
                .flatten()
                .filter(move |&point| {
                    !(0..self.blizzards.len()).any(|blizzard| {
                        if let Some(point) = point {
                            self.position_at_time(blizzard, node.time + 1) == point
                        } else {
                            // Blizzards never arrive at the start point
                            false
                        }
                    })
                })
                .map(move |point| Node {
                    position: point,
                    time: node.time + 1,
                })
        };

        let mut open_set = BinaryHeap::new();
        open_set.push(Reverse(WeightedNode {
            data: Node {
                position: None,
                time: 0,
            },
            cost: heuristic(None),
        }));

        let mut g_score = HashMap::new();
        g_score.insert(
            Node {
                position: None,
                time: 0,
            },
            0,
        );

        let mut f_score = HashMap::new();
        f_score.insert(
            Node {
                position: None,
                time: 0,
            },
            heuristic(None),
        );

        while let Some(Reverse(current)) = open_set.pop() {
            if let Some(position) = current.data.position {
                if position == end {
                    return Some(current.cost + 1);
                }
            }

            for neighbour in neighbours(current.data) {
                let tentative_g_score = g_score[&current.data] + 1;
                if tentative_g_score < *g_score.get(&neighbour).unwrap_or(&u32::MAX) {
                    g_score.insert(neighbour, tentative_g_score);
                    f_score.insert(neighbour, tentative_g_score + heuristic(neighbour.position));
                    open_set.push(Reverse(WeightedNode {
                        data: neighbour,
                        cost: f_score[&neighbour],
                    }));
                }
            }
        }

        None
    }

    fn position_at_time(&self, blizzard: usize, time: u32) -> Point2<u8> {
        let blizzard = self.blizzards[blizzard];

        let time: i64 = time.into();
        let delta: Vector2<i64> = match blizzard.direction {
            Direction::Up => Vector2::new(0, -1),
            Direction::Down => Vector2::new(0, 1),
            Direction::Left => Vector2::new(-1, 0),
            Direction::Right => Vector2::new(1, 0),
        };
        let delta = delta * time;

        let initial_position = Point2::new(blizzard.initial.x.into(), blizzard.initial.y.into());

        let new_position = initial_position + delta;
        let new_position = Point2::new(
            new_position.x.rem_euclid(self.width.into()),
            new_position.y.rem_euclid(self.height.into()),
        );

        Point2::new(
            new_position.x.try_into().unwrap(),
            new_position.y.try_into().unwrap(),
        )
    }
}

impl TryFrom<char> for Direction {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '^' => Ok(Self::Up),
            'v' => Ok(Self::Down),
            '<' => Ok(Self::Left),
            '>' => Ok(Self::Right),
            _ => bail!("Invalid direction {}", c),
        }
    }
}

fn parse_input() -> Result<Map> {
    let lines = input_lines()?;

    let mut blizzards = vec![];
    let mut max_x = 0;
    let mut max_y = 0;

    for (y, line) in lines.iter().enumerate() {
        if y == 0 {
            if !line.starts_with("#.") {
                bail!("Invalid first line {}", line.clone());
            }
            continue;
        } else if y == lines.len() - 1 {
            if !line.ends_with(".#") {
                bail!("Invalid last line {}", line.clone());
            }
            continue;
        }

        // The actual map we're interested in doesn't contain the first and last lines
        let y = y - 1;

        max_y = max_y.max(y);

        let mut line_done = false;
        for (x, element) in line.chars().skip(1).enumerate() {
            if line_done {
                bail!("Encountered wall in the middle of the line");
            }
            if element == '#' {
                line_done = true;
                continue;
            }

            max_x = max_x.max(x);

            if element == '.' {
                continue;
            }

            let direction = element.try_into()?;
            let position = Point2::new(x.try_into()?, y.try_into()?);
            let blizzard = Blizzard {
                initial: position,
                direction,
            };
            blizzards.push(blizzard);
        }
    }

    let map = Map {
        blizzards,
        width: (max_x + 1).try_into()?,
        height: (max_y + 1).try_into()?,
    };

    Ok(map)
}

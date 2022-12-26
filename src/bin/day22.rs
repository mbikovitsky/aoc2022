use std::fmt::Display;

use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::{Either, Itertools};
use ndarray::Array2;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map_res},
    multi::many1,
    IResult,
};

fn main() -> Result<()> {
    let (map, instructions) = parse_input();

    let password = map.simulate(instructions.iter().copied());
    dbg!(password);

    Ok(())
}

#[derive(Debug, Clone)]
struct Map {
    tiles: Array2<Tile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    Void,
    Open,
    Wall,
}

impl Default for Tile {
    fn default() -> Self {
        Self::Void
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Instruction {
    Move(u8),
    TurnCW,
    TurnCCW,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

impl Map {
    fn simulate(&self, instructions: impl IntoIterator<Item = Instruction>) -> u32 {
        let mut row = 0;
        let mut column = self
            .tiles
            .row(0)
            .indexed_iter()
            .find_map(|(x, tile)| if let Tile::Open = tile { Some(x) } else { None })
            .unwrap();
        let mut facing = Direction::Right;

        for instruction in instructions {
            match instruction {
                Instruction::Move(steps) => {
                    let axis = match facing {
                        Direction::Right | Direction::Left => self.tiles.row(row),
                        Direction::Down | Direction::Up => self.tiles.column(column),
                    };

                    let sliced = match facing {
                        Direction::Right => Either::Left(column..self.tiles.shape()[1]),
                        Direction::Down => Either::Left(row..self.tiles.shape()[0]),
                        Direction::Left => Either::Right((0..=column).rev()),
                        Direction::Up => Either::Right((0..=row).rev()),
                    };
                    let full = match facing {
                        Direction::Right => Either::Left(0..self.tiles.shape()[1]),
                        Direction::Down => Either::Left(0..self.tiles.shape()[0]),
                        Direction::Left => Either::Right((0..self.tiles.shape()[1]).rev()),
                        Direction::Up => Either::Right((0..self.tiles.shape()[0]).rev()),
                    };
                    let indices = sliced.chain(full.cycle());
                    let indices = indices.filter(|&index| !matches!(axis[index], Tile::Void));
                    let indices = indices.take_while(|&index| !matches!(axis[index], Tile::Wall));

                    let new_index = indices.take((steps + 1).into()).last().unwrap();

                    match facing {
                        Direction::Right | Direction::Left => column = new_index,
                        Direction::Down | Direction::Up => row = new_index,
                    }
                }
                Instruction::TurnCW => {
                    facing = match facing {
                        Direction::Right => Direction::Down,
                        Direction::Down => Direction::Left,
                        Direction::Left => Direction::Up,
                        Direction::Up => Direction::Right,
                    }
                }
                Instruction::TurnCCW => {
                    facing = match facing {
                        Direction::Right => Direction::Up,
                        Direction::Down => Direction::Right,
                        Direction::Left => Direction::Down,
                        Direction::Up => Direction::Left,
                    }
                }
            }
        }

        let row: u32 = row.try_into().unwrap();
        let column: u32 = column.try_into().unwrap();
        1000 * (row + 1) + 4 * (column + 1) + facing as u32
    }
}

impl TryFrom<char> for Tile {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            ' ' => Ok(Self::Void),
            '.' => Ok(Self::Open),
            '#' => Ok(Self::Wall),
            _ => bail!("Unknown tile type {}", value),
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.tiles.rows() {
            for tile in row {
                write!(f, "{}", tile)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Void => ' ',
            Tile::Open => '.',
            Tile::Wall => '#',
        };
        write!(f, "{}", c)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move(steps) => write!(f, "{}", steps),
            Self::TurnCW => write!(f, "R"),
            Self::TurnCCW => write!(f, "L"),
        }
    }
}

fn parse_input() -> (Map, Vec<Instruction>) {
    let lines = input_lines().unwrap();
    let (map_lines, instructions_line) = lines
        .split(|line| line.is_empty())
        .collect_tuple()
        .expect("Expected two sections in input file");

    assert_eq!(instructions_line.len(), 1);
    let instructions_line = &instructions_line[0];

    let map_width = map_lines
        .iter()
        .max_by_key(|line| line.len())
        .map_or(0, |line| line.len());
    let map_height = map_lines.len();

    let mut tiles = Array2::default((map_height, map_width));
    for (y, line) in map_lines.iter().enumerate() {
        for (x, tile) in line.char_indices() {
            tiles[(y, x)] = tile.try_into().unwrap();
        }
    }

    fn parse_move(input: &str) -> IResult<&str, Instruction> {
        map_res(digit1, |s: &str| -> Result<Instruction> {
            Ok(Instruction::Move(s.parse()?))
        })(input)
    }

    fn parse_ccw(input: &str) -> IResult<&str, Instruction> {
        map_res(tag("L"), |_| -> Result<Instruction> {
            Ok(Instruction::TurnCCW)
        })(input)
    }

    fn parse_cw(input: &str) -> IResult<&str, Instruction> {
        map_res(tag("R"), |_| -> Result<Instruction> {
            Ok(Instruction::TurnCW)
        })(input)
    }

    let (_, instructions) =
        all_consuming(many1(alt((parse_move, parse_cw, parse_ccw))))(instructions_line)
            .expect("Extraneous characters in instructions line");

    (Map { tiles }, instructions)
}

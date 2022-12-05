use std::collections::VecDeque;

use anyhow::{bail, Context, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

fn main() -> Result<()> {
    let (mut stacks, instructions) = parse_input()?;

    let mut new_stacks = stacks.clone();
    for instruction in &instructions {
        new_stacks.move_crates_one_by_one(
            instruction.count.into(),
            (instruction.from - 1).into(),
            (instruction.to - 1).into(),
        );
    }
    for (_, crat) in new_stacks.top_crates() {
        print!("{}", crat as char);
    }
    println!();

    for instruction in &instructions {
        stacks.move_crates_in_bulk(
            instruction.count.into(),
            (instruction.from - 1).into(),
            (instruction.to - 1).into(),
        );
    }
    for (_, crat) in stacks.top_crates() {
        print!("{}", crat as char);
    }
    println!();

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Stacks {
    stacks: Vec<VecDeque<u8>>,
}

impl Stacks {
    fn move_crates_one_by_one(&mut self, count: usize, from: usize, to: usize) {
        for _ in 0..count {
            if let Some(element) = self.stacks[from].pop_front() {
                self.stacks[to].push_front(element)
            }
        }
    }

    fn move_crates_in_bulk(&mut self, count: usize, from: usize, to: usize) {
        let to_move = self.stacks[from].drain(..count).rev().collect_vec();
        for element in to_move {
            self.stacks[to].push_front(element);
        }
    }

    fn top_crates(&self) -> impl Iterator<Item = (usize, u8)> + '_ {
        self.stacks.iter().enumerate().filter_map(|(index, stack)| {
            if stack.is_empty() {
                None
            } else {
                Some((index, stack[0]))
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Instruction {
    count: u8,
    from: u8,
    to: u8,
}

fn parse_input() -> Result<(Stacks, Vec<Instruction>)> {
    let lines = input_lines()?;

    let (initial_state, instructions) = lines
        .split(|line| line.is_empty())
        .collect_tuple()
        .context("Invalid file format")?;

    let mut stacks = vec![];
    for (line_index, line) in initial_state.iter().enumerate() {
        if !line.is_ascii() {
            bail!("Expected ASCII string, got {}", line);
        }

        let line = line.as_bytes();

        let mut char_index = 0;
        let mut column_index = 0;
        while char_index < line.len() {
            assert!(column_index <= stacks.len());
            if column_index == stacks.len() {
                stacks.push(VecDeque::new());
            }

            let element = &line[char_index..char_index + 3];

            match element {
                [b'[', e @ b'A'..=b'Z', b']'] => {
                    if line_index == initial_state.len() - 1 {
                        bail!("Didn't expect a full column at the bottom");
                    }

                    stacks[column_index].push_back(*e);
                }
                [b' ', b' ', b' '] => {
                    // Empty column
                    if line_index == initial_state.len() - 1 {
                        bail!("Didn't expect an empty column at the bottom");
                    }
                }
                [b' ', b'1'..=b'9', b' '] => {
                    // Numbering row
                    if line_index != initial_state.len() - 1 {
                        bail!("Expected numbering row to be at the very end")
                    }
                }
                _ => bail!("Invalid element"),
            }

            // Skip the current element and the separator
            char_index += 4;

            column_index += 1;
        }
    }

    lazy_static! {
        static ref INSTRUCTION_REGEX: Regex =
            Regex::new(r#"^move (\d+) from (\d+) to (\d+)$"#).unwrap();
    }

    let instructions = instructions
        .iter()
        .map(|instruction| {
            let captures = INSTRUCTION_REGEX
                .captures(instruction)
                .context("Invalid instruction")?;

            let count: u8 = captures[1].parse().context("Invalid move count")?;
            let from: u8 = captures[2].parse().context("Invalid move count")?;
            let to: u8 = captures[3].parse().context("Invalid move count")?;

            Ok(Instruction { count, from, to })
        })
        .collect::<Result<Vec<Instruction>>>()?;

    Ok((Stacks { stacks }, instructions))
}

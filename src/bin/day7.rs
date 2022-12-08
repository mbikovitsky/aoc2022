use std::{borrow::Borrow, collections::HashMap, iter, str::FromStr};

use anyhow::{bail, Context, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;
use lazy_static::lazy_static;
use petgraph::{
    matrix_graph::{DiMatrix, NodeIndex},
    visit::DfsPostOrder,
};
use regex::Regex;

fn main() -> Result<()> {
    let tree = parse_input()?;

    let sum_of_sizes: u32 = tree.directory_sizes().filter(|&size| size <= 100_000).sum();
    dbg!(sum_of_sizes);

    let directory_sizes = tree.directory_sizes().sorted().collect_vec();

    const DISK_SIZE: u32 = 70_000_000;
    const REQUIRED_FREE_SPACE: u32 = 30_000_000;

    // The largest directory is the root, since it contains everything else
    let used_space = *directory_sizes.last().unwrap();

    let free_space = DISK_SIZE - used_space;

    let size_to_free = directory_sizes
        .iter()
        .copied()
        .find(|&size| free_space + size >= REQUIRED_FREE_SPACE)
        .context("Didn't find a directory to free")?;
    dbg!(size_to_free);

    Ok(())
}

fn parse_input() -> Result<Fs> {
    let lines = input_lines()?
        .into_iter()
        .map(|line| line.parse())
        .collect::<Result<Vec<Line>>>()?;

    Fs::parse(lines)
}

#[derive(Clone)]
struct Fs {
    root: NodeIndex<u16>,
    graph: DiMatrix<FsObject, ()>,
}

#[derive(Debug, Clone)]
enum FsObject {
    Directory {
        name: String,
    },
    File {
        #[allow(unused)]
        name: String,
        size: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Line {
    Instruction(Instruction),
    Directory { name: String },
    File { name: String, size: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Instruction {
    ChangeDirectory(String),
    ListDirectory,
}

impl Fs {
    fn directory_sizes(&self) -> impl Iterator<Item = u32> + '_ {
        let mut post_order = DfsPostOrder::new(&self.graph, self.root);
        let mut sizes = HashMap::<NodeIndex<u16>, u32>::new();
        iter::from_fn(move || post_order.next(&self.graph)).filter_map(move |node| {
            match self.graph.node_weight(node) {
                FsObject::Directory { .. } => {
                    let size = self
                        .graph
                        .neighbors(node)
                        .map(|neighbour| match self.graph.node_weight(neighbour) {
                            FsObject::Directory { .. } => sizes[&neighbour],
                            FsObject::File { size, .. } => *size,
                        })
                        .sum();
                    sizes.insert(node, size);
                    Some(size)
                }
                FsObject::File { .. } => None,
            }
        })
    }

    fn parse(lines: impl IntoIterator<Item = impl Borrow<Line>>) -> Result<Self> {
        let mut graph = DiMatrix::new();

        let root = graph.add_node(FsObject::Directory {
            name: "/".to_owned(),
        });

        let mut current_path = vec![root];

        for line in lines {
            match line.borrow() {
                Line::Instruction(Instruction::ListDirectory) => {}
                Line::Instruction(Instruction::ChangeDirectory(new_dir_name)) => {
                    match new_dir_name.as_str() {
                        "/" => {
                            current_path.drain(1..);
                        }
                        ".." => {
                            current_path.pop().context("Attempting to cd past /")?;
                        }
                        _ => {
                            let current_dir = *current_path.last().unwrap();

                            if let Some(new_directory) =
                                graph.neighbors(current_dir).find(|&neighbour| {
                                    match graph.node_weight(neighbour) {
                                        FsObject::Directory { name } => name == new_dir_name,
                                        _ => false,
                                    }
                                })
                            {
                                current_path.push(new_directory);
                            } else {
                                bail!("Expected current directory to contain {}", new_dir_name);
                            }
                        }
                    }
                }
                Line::Directory { name } => {
                    let node = graph.add_node(FsObject::Directory { name: name.clone() });
                    graph.add_edge(*current_path.last().unwrap(), node, ());
                }
                Line::File { name, size } => {
                    let node = graph.add_node(FsObject::File {
                        name: name.clone(),
                        size: *size,
                    });
                    graph.add_edge(*current_path.last().unwrap(), node, ());
                }
            }
        }

        Ok(Self { root, graph })
    }
}

impl FromStr for Line {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref INSTRUCTION_REGEX: Regex = Regex::new(r#"^\$ (cd|ls)(?: (.+)|)$"#).unwrap();
            static ref DIR_REGEX: Regex = Regex::new(r#"^dir (.+)$"#).unwrap();
            static ref FILE_REGEX: Regex = Regex::new(r#"^(\d+) (.+)$"#).unwrap();
        }

        if let Some(captures) = INSTRUCTION_REGEX.captures(s) {
            let command = captures.get(1).unwrap().as_str();
            match command {
                "cd" => {
                    if let Some(argument) = captures.get(2) {
                        Ok(Self::Instruction(Instruction::ChangeDirectory(
                            argument.as_str().to_owned(),
                        )))
                    } else {
                        bail!("Expected argument to 'cd' command");
                    }
                }
                "ls" => Ok(Self::Instruction(Instruction::ListDirectory)),
                _ => bail!("Unknown command '{}'", command),
            }
        } else if let Some(captures) = DIR_REGEX.captures(s) {
            Ok(Self::Directory {
                name: captures.get(1).unwrap().as_str().to_owned(),
            })
        } else if let Some(captures) = FILE_REGEX.captures(s) {
            Ok(Self::File {
                name: captures.get(2).unwrap().as_str().to_owned(),
                size: captures.get(1).unwrap().as_str().parse()?,
            })
        } else {
            bail!("Invalid line {}", s);
        }
    }
}

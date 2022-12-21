use std::{collections::HashMap, str::FromStr};

use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;
use lazy_static::lazy_static;
use petgraph::{graph::DiGraph, visit::DfsPostOrder, Direction};
use regex::Regex;

fn main() -> Result<()> {
    let monkeys = parse_input()?;

    let result = compute(&monkeys, "root");
    dbg!(result);

    Ok(())
}

fn compute(monkeys: &[Monkey], start: &str) -> i64 {
    let mut graph = DiGraph::new();
    let mut name_to_node_index = HashMap::new();

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Node {
        Value(i64),
        Op(Operation),
    }

    for monkey in monkeys {
        let node = match monkey.job {
            Job::Const(value) => Node::Value(value),
            Job::Compute { op, .. } => Node::Op(op),
        };
        let node_index = graph.add_node(node);
        let inserted = name_to_node_index
            .insert(monkey.name.as_str(), node_index)
            .is_none();
        assert!(inserted);
    }

    for monkey in monkeys {
        if let Job::Compute { left, right, .. } = &monkey.job {
            let current = name_to_node_index[monkey.name.as_str()];
            let left = name_to_node_index[left.as_str()];
            let right = name_to_node_index[right.as_str()];
            // For a directed graph, neighbors_directed returns nodes in reverse order
            // of insertion
            graph.add_edge(current, right, ());
            graph.add_edge(current, left, ());
        }
    }

    let mut dfs = DfsPostOrder::new(&graph, name_to_node_index[start]);
    while let Some(node) = dfs.next(&graph) {
        let computed = match graph.node_weight(node).unwrap() {
            Node::Value(value) => *value,
            Node::Op(op) => {
                let (left, right) = graph
                    .neighbors_directed(node, Direction::Outgoing)
                    .collect_tuple()
                    .expect("Expected exactly two neighbours");
                let left = if let Node::Value(value) = graph.node_weight(left).unwrap() {
                    *value
                } else {
                    panic!("Uncomputed left neighbour");
                };
                let right = if let Node::Value(value) = graph.node_weight(right).unwrap() {
                    *value
                } else {
                    panic!("Uncomputed right neighbour");
                };

                match op {
                    Operation::Add => left + right,
                    Operation::Sub => left - right,
                    Operation::Mul => left * right,
                    Operation::Div => left / right,
                }
            }
        };

        *graph.node_weight_mut(node).unwrap() = Node::Value(computed);
    }

    match graph.node_weight(name_to_node_index[start]).unwrap() {
        Node::Value(value) => *value,
        _ => panic!("Didn't compute node"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Monkey {
    name: String,
    job: Job,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Job {
    Const(i64),
    Compute {
        op: Operation,
        left: String,
        right: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl FromStr for Operation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            "/" => Ok(Self::Div),
            _ => bail!("Invalid operation {}", s),
        }
    }
}

fn parse_input() -> Result<Vec<Monkey>> {
    input_lines()?
        .into_iter()
        .map(|line| {
            lazy_static! {
                static ref REGEX: Regex =
                    Regex::new(r#"^(\w+): ((\w+) ([+\-*/]) (\w+)|\d+)$"#).unwrap();
            }

            if let Some(captures) = REGEX.captures(&line) {
                let name = captures.get(1).unwrap().as_str();

                let job = if let Some(left) = captures.get(3) {
                    let left = left.as_str();
                    let right = captures.get(5).unwrap().as_str();
                    let op = captures.get(4).unwrap().as_str().parse()?;

                    Job::Compute {
                        op,
                        left: left.to_owned(),
                        right: right.to_owned(),
                    }
                } else {
                    let value = captures.get(2).unwrap().as_str().parse()?;

                    Job::Const(value)
                };

                Ok(Monkey {
                    name: name.to_owned(),
                    job,
                })
            } else {
                bail!("Invalid descriptor {}", line);
            }
        })
        .collect()
}

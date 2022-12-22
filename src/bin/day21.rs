use std::{collections::HashMap, str::FromStr};

use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;
use lazy_static::lazy_static;
use petgraph::{
    graph::DiGraph,
    visit::{Dfs, DfsPostOrder},
    Direction,
};
use regex::Regex;

fn main() -> Result<()> {
    let monkeys = parse_input()?;

    let result = compute(&monkeys, "root");
    dbg!(result);

    let unknown = compute_unknown(&monkeys, "root", "humn");
    dbg!(unknown);

    Ok(())
}

fn compute(monkeys: &[Monkey], root: &str) -> i64 {
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

    let mut dfs = DfsPostOrder::new(&graph, name_to_node_index[root]);
    while let Some(node) = dfs.next(&graph) {
        let computed = match &graph[node] {
            Node::Value(value) => *value,
            Node::Op(op) => {
                let (left, right) = graph
                    .neighbors_directed(node, Direction::Outgoing)
                    .collect_tuple()
                    .expect("Expected exactly two neighbours");
                let left = if let Node::Value(value) = &graph[left] {
                    *value
                } else {
                    panic!("Uncomputed left neighbour");
                };
                let right = if let Node::Value(value) = &graph[right] {
                    *value
                } else {
                    panic!("Uncomputed right neighbour");
                };

                match op {
                    Operation::Add => left + right,
                    Operation::Sub => left - right,
                    Operation::Mul => left * right,
                    Operation::Div => left / right,
                    Operation::Eq => panic!("There shouldn't be an = operator"),
                }
            }
        };

        graph[node] = Node::Value(computed);
    }

    match &graph[name_to_node_index[root]] {
        Node::Value(value) => *value,
        _ => panic!("Didn't compute node"),
    }
}

fn compute_unknown(monkeys: &[Monkey], root: &str, unknown: &str) -> i64 {
    let mut graph = DiGraph::new();
    let mut name_to_node_index = HashMap::new();

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Node {
        value: Option<i64>,
        op: Option<Operation>,
    }

    for monkey in monkeys {
        let node = if monkey.name == root {
            Node {
                value: None,
                op: Some(Operation::Eq),
            }
        } else if monkey.name == unknown {
            Node {
                value: None,
                op: None,
            }
        } else {
            match monkey.job {
                Job::Const(value) => Node {
                    value: Some(value),
                    op: None,
                },
                Job::Compute { op, .. } => Node {
                    value: None,
                    op: Some(op),
                },
            }
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

    // 1st pass: traverse the tree bottom-up, and compute as much as possible

    let mut dfs = DfsPostOrder::new(&graph, name_to_node_index[root]);
    while let Some(node) = dfs.next(&graph) {
        if graph[node].op.is_none() {
            continue;
        }

        assert!(
            graph[node].value.is_none(),
            "A compute node shouldn't have an initial value"
        );

        let (left, right) = graph
            .neighbors_directed(node, Direction::Outgoing)
            .collect_tuple()
            .expect("Expected exactly two neighbours");

        let computed = match (graph[left].value, graph[right].value) {
            (None, None) => panic!("Detected operation with both branches unknown"),

            (Some(left), Some(right)) => match graph[node].op.unwrap() {
                Operation::Add => Some(left + right),
                Operation::Sub => Some(left - right),
                Operation::Mul => Some(left * right),
                Operation::Div => Some(left / right),
                Operation::Eq => {
                    // Skip the root for now
                    assert!(node == name_to_node_index[root]);
                    None
                }
            },

            // One of the branches is unknown, so skip for now
            (None, Some(_)) => None,
            (Some(_), None) => None,
        };

        graph[node].value = computed;
    }

    // 2nd pass: traverse top-down to find the unknown value

    assert!(graph[name_to_node_index[root]].value.is_none());
    assert!(graph[name_to_node_index[root]].op == Some(Operation::Eq));

    let mut dfs = Dfs::new(&graph, name_to_node_index[root]);
    while let Some(node) = dfs.next(&graph) {
        if graph[node].op.is_none() {
            continue;
        }

        let (left, right) = graph
            .neighbors_directed(node, Direction::Outgoing)
            .collect_tuple()
            .expect("Expected exactly two neighbours");

        if graph[left].value.is_some() && graph[right].value.is_some() {
            continue;
        }

        let computed = solve_for_x(
            graph[node].op.unwrap(),
            graph[left].value,
            graph[right].value,
            graph[node].value,
        );

        if graph[left].value.is_none() {
            graph[left].value = Some(computed);
        } else {
            graph[right].value = Some(computed);
        }
    }

    graph[name_to_node_index[unknown]]
        .value
        .expect("Didn't compute the unknown value")
}

fn solve_for_x(op: Operation, left: Option<i64>, right: Option<i64>, expected: Option<i64>) -> i64 {
    if left.is_none() && right.is_none() {
        panic!("Both operands are unknown");
    } else if left.is_some() && right.is_some() {
        panic!("Both operands are known");
    } else {
        match op {
            Operation::Add => expected.unwrap() - left.unwrap_or_else(|| right.unwrap()),
            Operation::Sub => {
                if let Some(left) = left {
                    left - expected.unwrap()
                } else {
                    expected.unwrap() + right.unwrap()
                }
            }
            Operation::Mul => expected.unwrap() / left.unwrap_or_else(|| right.unwrap()),
            Operation::Div => {
                if let Some(left) = left {
                    left / expected.unwrap()
                } else {
                    expected.unwrap() * right.unwrap()
                }
            }
            Operation::Eq => left.unwrap_or_else(|| right.unwrap()),
        }
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
    Eq,
}

impl FromStr for Operation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            "/" => Ok(Self::Div),
            "=" => Ok(Self::Eq),
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

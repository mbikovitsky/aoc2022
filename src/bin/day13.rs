use std::{fmt::Display, str::FromStr};

use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map_res},
    multi::separated_list0,
    sequence::delimited,
    IResult,
};

fn main() -> Result<()> {
    let packet_pairs = parse_input();

    let mut sum_of_indices = 0;
    for (index, (left, right)) in packet_pairs.iter().enumerate() {
        if left < right {
            sum_of_indices += index + 1;
        }
    }
    dbg!(sum_of_indices);

    let divider1: Packet = "[[2]]".parse().unwrap();
    let divider2: Packet = "[[6]]".parse().unwrap();

    let packets = packet_pairs
        .into_iter()
        .chain(std::iter::once((divider1.clone(), divider2.clone())))
        .flat_map(|(a, b)| [a, b])
        .sorted()
        .collect_vec();

    let divider1_index = packets.binary_search(&divider1).unwrap() + 1;
    let divider2_index = packets.binary_search(&divider2).unwrap() + 1;
    dbg!(divider1_index * divider2_index);

    Ok(())
}

fn parse_input() -> Vec<(Packet, Packet)> {
    input_lines()
        .unwrap()
        .split(String::is_empty)
        .map(|lines| {
            assert_eq!(lines.len(), 2);

            let first = lines[0].parse().unwrap();
            let second = lines[1].parse().unwrap();

            (first, second)
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Packet {
    contents: Vec<PacketElement>,
}

#[derive(Debug, Clone)]
enum PacketElement {
    List(Vec<PacketElement>),
    Int(u32),
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        if let Some((last, first)) = self.contents.split_last() {
            for element in first {
                write!(f, "{},", element)?;
            }
            last.fmt(f)?;
        }

        write!(f, "]")?;

        Ok(())
    }
}

impl Display for PacketElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(list) => {
                write!(f, "[")?;

                if let Some((last, first)) = list.split_last() {
                    for element in first {
                        write!(f, "{},", element)?;
                    }
                    last.fmt(f)?;
                }

                write!(f, "]")?;

                Ok(())
            }
            Self::Int(int) => write!(f, "{}", int),
        }
    }
}

impl PartialEq for PacketElement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(lhs), Self::Int(rhs)) => lhs.eq(rhs),
            (Self::List(lhs), Self::List(rhs)) => lhs.eq(rhs),

            (Self::List(_), Self::Int(rhs)) => self.eq(&Self::List(vec![Self::Int(*rhs)])),
            (Self::Int(lhs), Self::List(_)) => Self::List(vec![Self::Int(*lhs)]).eq(other),
        }
    }
}

impl Eq for PacketElement {}

impl PartialOrd for PacketElement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Int(lhs), Self::Int(rhs)) => lhs.partial_cmp(rhs),
            (Self::List(lhs), Self::List(rhs)) => lhs.partial_cmp(rhs),

            (Self::List(_), Self::Int(rhs)) => self.partial_cmp(&Self::List(vec![Self::Int(*rhs)])),
            (Self::Int(lhs), Self::List(_)) => Self::List(vec![Self::Int(*lhs)]).partial_cmp(other),
        }
    }
}

impl Ord for PacketElement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl FromStr for Packet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let element: PacketElement = s.parse()?;
        match element {
            PacketElement::List(list) => Ok(Self { contents: list }),
            PacketElement::Int(int) => bail!("Expected list, found int: {}", int),
        }
    }
}

impl FromStr for PacketElement {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_element(input: &str) -> IResult<&str, PacketElement> {
            let parse_int = map_res(digit1, |s: &str| -> anyhow::Result<PacketElement> {
                Ok(PacketElement::Int(s.parse()?))
            });

            let parse_list = map_res(
                delimited(tag("["), separated_list0(tag(","), parse_element), tag("]")),
                |list| -> anyhow::Result<PacketElement> { Ok(PacketElement::List(list)) },
            );

            alt((parse_int, parse_list))(input)
        }

        match all_consuming(parse_element)(s) {
            Ok((_, element)) => Ok(element),
            Err(error) => Err(error.to_owned().into()),
        }
    }
}

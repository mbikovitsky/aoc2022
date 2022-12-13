mod packet;
lalrpop_mod!(packet_parser, "/bin/day13/packet.rs");

use anyhow::Result;
use aoc2022::util::input_lines;
use itertools::Itertools;
use lalrpop_util::lalrpop_mod;
use packet::Packet;
use packet_parser::PacketParser;

fn main() -> Result<()> {
    let packet_pairs = parse_input();

    let mut sum_of_indices = 0;
    for (index, (left, right)) in packet_pairs.iter().enumerate() {
        if left < right {
            sum_of_indices += index + 1;
        }
    }
    dbg!(sum_of_indices);

    let divider1 = PacketParser::new().parse("[[2]]").unwrap();
    let divider2 = PacketParser::new().parse("[[6]]").unwrap();

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

            let parser = PacketParser::new();
            let first = parser.parse(&lines[0]).unwrap();
            let second = parser.parse(&lines[1]).unwrap();

            (first, second)
        })
        .collect()
}

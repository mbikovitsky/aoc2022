use std::iter;

use anyhow::Result;
use aoc2022::util::input_lines;
use itertools::Itertools;

const DECRYPTION_KEY: i64 = 811589153;

fn main() -> Result<()> {
    let numbers = parse_input()?;

    let (a, b, c) = compute_coordinates(&numbers, 1);
    dbg!(a + b + c);

    let mut numbers = numbers;
    for number in numbers.iter_mut() {
        *number *= DECRYPTION_KEY;
    }
    let (a, b, c) = compute_coordinates(&numbers, 10);
    dbg!(a + b + c);

    Ok(())
}

fn compute_coordinates(numbers: &[i64], rounds: u32) -> (i64, i64, i64) {
    let mut mixed_numbers = mix(numbers, rounds).cycle();

    mixed_numbers.find(|&value| value == 0).unwrap();

    let first = mixed_numbers.nth(1000 - 1).unwrap();
    let second = mixed_numbers.nth(1000 - 1).unwrap();
    let third = mixed_numbers.nth(1000 - 1).unwrap();

    (first, second, third)
}

fn mix(numbers: &[i64], rounds: u32) -> impl Iterator<Item = i64> + Clone {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Entry {
        value: i64,
        next: usize,
        prev: usize,
    }

    let mut list = numbers
        .iter()
        .enumerate()
        .map(|(index, &number)| Entry {
            value: number,
            next: if index == numbers.len() - 1 {
                0
            } else {
                index + 1
            },
            prev: if index == 0 {
                numbers.len() - 1
            } else {
                index - 1
            },
        })
        .collect_vec();

    fn next(list: &[Entry], entry: usize, count: usize) -> usize {
        let mut remaining = count;
        let mut current = entry;
        while remaining > 0 {
            current = list[current].next;
            remaining -= 1;
        }
        current
    }

    fn prev(list: &[Entry], entry: usize, count: usize) -> usize {
        let mut remaining = count;
        let mut current = entry;
        while remaining > 0 {
            current = list[current].prev;
            remaining -= 1;
        }
        current
    }

    fn insert_after(list: &mut [Entry], entry: usize, prev: usize) {
        assert_ne!(entry, prev);

        let next = list[prev].next;

        list[entry].next = next;
        list[entry].prev = prev;

        list[prev].next = entry;
        list[next].prev = entry;
    }

    fn remove(list: &mut [Entry], entry: usize) {
        let next = list[entry].next;
        let prev = list[entry].prev;
        list[prev].next = next;
        list[next].prev = prev;
    }

    for _ in 0..rounds {
        for entry in 0..list.len() {
            remove(&mut list, entry);

            let after = if list[entry].value < 0 {
                let count: usize = list[entry].value.checked_neg().unwrap().try_into().unwrap();
                prev(&list, list[entry].prev, count % (list.len() - 1))
            } else {
                let count: usize = list[entry].value.try_into().unwrap();
                next(&list, list[entry].prev, count % (list.len() - 1))
            };

            insert_after(&mut list, entry, after);
        }
    }

    let mut remaining = list.len();
    let mut current = 0;
    iter::from_fn(move || {
        if remaining == 0 {
            None
        } else {
            let value = list[current].value;
            current = list[current].next;
            remaining -= 1;
            Some(value)
        }
    })
}

fn parse_input() -> Result<Vec<i64>> {
    input_lines()?.into_iter().map(|s| Ok(s.parse()?)).collect()
}

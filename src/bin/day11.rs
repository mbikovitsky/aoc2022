use std::{
    cmp::Reverse,
    num::NonZeroU32,
    ops::{AddAssign, DivAssign, MulAssign},
};

use anyhow::{bail, Context, Result};
use aoc2022::{
    galois::{GFInt, GF},
    util::input_lines,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

fn main() -> Result<()> {
    let mut monkeys = parse_input()?;

    let monkey_business = simulate(&mut monkeys.clone(), NonZeroU32::new(3).unwrap(), 20);
    dbg!(monkey_business);

    let divisors = monkeys
        .iter()
        .map(|monkey| monkey.test)
        .unique()
        .collect_vec();
    for monkey in monkeys.iter_mut() {
        for item in monkey.items.iter_mut() {
            // There should only be one item, without any special handling
            assert_eq!(item.values.len(), 1);
            assert!(item.values[0].field().order().is_none());

            let value = item.values[0].value();

            let new_values = divisors
                .iter()
                .map(|divisor| GF::new(Some(*divisor)).create_value(value))
                .collect();

            item.values = new_values;
        }
    }

    let monkey_business = simulate(&mut monkeys, NonZeroU32::new(1).unwrap(), 10000);
    dbg!(monkey_business);

    Ok(())
}

#[derive(Debug, Clone)]
struct Monkey {
    id: usize,
    items: Vec<Item>,
    operation: Operation,
    test: NonZeroU32,
    target_true: usize,
    target_false: usize,
    inspections: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    MulConst(u32),
    AddConst(u32),
    MulSelf,
}

#[derive(Debug, Clone)]
struct Item {
    values: Vec<GFInt>,
}

impl Item {
    fn square_assign(&mut self) {
        for value in self.values.iter_mut() {
            value.square_assign();
        }
    }
}

impl AddAssign<u32> for Item {
    fn add_assign(&mut self, rhs: u32) {
        for value in self.values.iter_mut() {
            *value += value.field().create_value(rhs);
        }
    }
}

impl MulAssign<u32> for Item {
    fn mul_assign(&mut self, rhs: u32) {
        for value in self.values.iter_mut() {
            *value *= value.field().create_value(rhs);
        }
    }
}

impl DivAssign<u32> for Item {
    fn div_assign(&mut self, rhs: u32) {
        for value in self.values.iter_mut() {
            *value = value
                .field()
                .create_value(value.value().checked_div(rhs).unwrap());
        }
    }
}

fn simulate(monkeys: &mut [Monkey], divide_by: NonZeroU32, iterations: u32) -> usize {
    for _ in 0..iterations {
        simulate1(monkeys, divide_by);
    }

    monkeys.sort_by_key(|monkey| Reverse(monkey.inspections));

    let first = {
        monkeys.select_nth_unstable_by_key(0, |monkey| Reverse(monkey.inspections));
        monkeys[0].inspections
    };

    let second = {
        monkeys.select_nth_unstable_by_key(1, |monkey| Reverse(monkey.inspections));
        monkeys[1].inspections
    };

    first * second
}

fn simulate1(monkeys: &mut [Monkey], divide_by: NonZeroU32) {
    for index in 0..monkeys.len() {
        let monkey = &monkeys[index];
        assert!(monkey.id == index);
        let replacement = Monkey {
            id: monkey.id,
            items: vec![],
            operation: monkey.operation,
            test: monkey.test,
            target_true: monkey.target_true,
            target_false: monkey.target_false,
            inspections: monkey.inspections,
        };
        let monkey = std::mem::replace(&mut monkeys[index], replacement);

        for mut item in monkey.items {
            match monkey.operation {
                Operation::MulConst(val) => item *= val,
                Operation::AddConst(val) => item += val,
                Operation::MulSelf => item.square_assign(),
            };
            item /= divide_by.get();

            let should_branch = if let Some(value) = item
                .values
                .iter()
                .find(|item| item.field().order() == Some(monkey.test))
            {
                value.value() == 0
            } else {
                assert_eq!(item.values.len(), 1);
                assert!(item.values[0].field().order().is_none());

                item.values[0].value() % monkey.test == 0
            };

            let target = if should_branch {
                monkey.target_true
            } else {
                monkey.target_false
            };

            monkeys[target].items.push(item);

            monkeys[index].inspections += 1;
        }
    }
}

fn parse_input() -> Result<Vec<Monkey>> {
    let monkeys: Result<Vec<Monkey>> = input_lines()?
        .split(String::is_empty)
        .map(|descriptor| {
            if descriptor.len() != 6 {
                bail!(
                    "Invalid number of lines in descriptor: {}",
                    descriptor.len()
                );
            }

            lazy_static! {
                static ref ID_REGEX: Regex = Regex::new(r#"^Monkey (\d+):$"#).unwrap();
            }
            let id = ID_REGEX
                .captures(&descriptor[0])
                .context("Invalid ID line format")?
                .get(1)
                .unwrap()
                .as_str()
                .parse()?;

            lazy_static! {
                static ref ITEMS_REGEX: Regex = Regex::new(r#"^  Starting items: (.+)$"#).unwrap();
            }
            let items = ITEMS_REGEX
                .captures(&descriptor[1])
                .context("Invalid starting items format")?
                .get(1)
                .unwrap()
                .as_str()
                .split(", ")
                .map(|s| {
                    Ok(Item {
                        values: vec![GF::new(None).create_value(s.parse()?)],
                    })
                })
                .collect::<Result<_>>()?;

            lazy_static! {
                static ref MUL_CONST_REGEX: Regex =
                    Regex::new(r#"^  Operation: new = old \* (\d+)$"#).unwrap();
                static ref ADD_CONST_REGEX: Regex =
                    Regex::new(r#"^  Operation: new = old \+ (\d+)$"#).unwrap();
                static ref MUL_SELF_REGEX: Regex =
                    Regex::new(r#"^  Operation: new = old \* old$"#).unwrap();
            }
            let operation = if let Some(mul_const) = MUL_CONST_REGEX.captures(&descriptor[2]) {
                Operation::MulConst(mul_const.get(1).unwrap().as_str().parse()?)
            } else if let Some(add_const) = ADD_CONST_REGEX.captures(&descriptor[2]) {
                Operation::AddConst(add_const.get(1).unwrap().as_str().parse()?)
            } else if MUL_SELF_REGEX.is_match(&descriptor[2]) {
                Operation::MulSelf
            } else {
                bail!("Invalid operation {}", &descriptor[2]);
            };

            lazy_static! {
                static ref TEST_REGEX: Regex =
                    Regex::new(r#"^  Test: divisible by (\d+)$"#).unwrap();
            }
            let test = TEST_REGEX
                .captures(&descriptor[3])
                .context("Invalid test expression")?
                .get(1)
                .unwrap()
                .as_str()
                .parse()?;

            lazy_static! {
                static ref IF_TRUE_REGEX: Regex =
                    Regex::new(r#"^    If true: throw to monkey (\d+)$"#).unwrap();
                static ref IF_FALSE_REGEX: Regex =
                    Regex::new(r#"^    If false: throw to monkey (\d+)$"#).unwrap();
            }
            let target_true = IF_TRUE_REGEX
                .captures(&descriptor[4])
                .context("Invalid 'true' target")?
                .get(1)
                .unwrap()
                .as_str()
                .parse()?;
            let target_false = IF_FALSE_REGEX
                .captures(&descriptor[5])
                .context("Invalid 'true' target")?
                .get(1)
                .unwrap()
                .as_str()
                .parse()?;

            Ok(Monkey {
                id,
                items,
                operation,
                test,
                target_true,
                target_false,
                inspections: 0,
            })
        })
        .collect();
    let monkeys = monkeys?;

    for (index, monkey) in monkeys.iter().enumerate() {
        if index != monkey.id {
            bail!("Monkeys are not ordered correctly");
        }
    }

    Ok(monkeys)
}

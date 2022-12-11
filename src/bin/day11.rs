use anyhow::{bail, Context, Result};
use aoc2022::util::input_lines;
use lazy_static::lazy_static;
use regex::Regex;

fn main() -> Result<()> {
    let mut monkeys = parse_input()?;

    for _ in 0..20 {
        simulate1(&mut monkeys, 3);
    }
    monkeys.sort_by_key(|monkey| monkey.inspections);
    let monkey_business =
        monkeys[monkeys.len() - 2].inspections * monkeys[monkeys.len() - 1].inspections;
    dbg!(monkey_business);

    monkeys.sort_by_key(|monkey| monkey.id);

    for _ in 0..10000 {
        simulate1(&mut monkeys, 1);
    }
    monkeys.sort_by_key(|monkey| monkey.inspections);
    let monkey_business =
        monkeys[monkeys.len() - 2].inspections * monkeys[monkeys.len() - 1].inspections;
    dbg!(monkey_business);

    Ok(())
}

#[derive(Debug, Clone)]
struct Monkey {
    id: usize,
    items: Vec<u32>,
    operation: Operation,
    test: u32,
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

fn simulate1(monkeys: &mut [Monkey], divide_by: u32) {
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

        for item in monkey.items {
            let item = match monkey.operation {
                Operation::MulConst(val) => item * val,
                Operation::AddConst(val) => item + val,
                Operation::MulSelf => item * item,
            };
            let item = item / divide_by;

            let target = if item % monkey.test == 0 {
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
                .map(|s| Ok(s.parse()?))
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

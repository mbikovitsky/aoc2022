use std::{
    fmt::Display,
    ops::{Add, AddAssign},
    str::FromStr,
};

use anyhow::{bail, Context, Result};
use aoc2022::util::input_lines;
use itertools::{EitherOrBoth, Itertools};

fn main() -> Result<()> {
    let numbers = parse_input()?;

    let sum = numbers
        .into_iter()
        .reduce(|acc, number| acc + number)
        .unwrap();
    println!("{}", sum);

    Ok(())
}

#[derive(Debug, Clone)]
struct SnafuInt {
    digits: Vec<i8>,
}

impl AddAssign<&Self> for SnafuInt {
    fn add_assign(&mut self, rhs: &Self) {
        if self.digits.len() < rhs.digits.len() {
            self.digits.resize(rhs.digits.len(), 0);
        }

        /// Adds two SNAFU digits.
        /// Returns a tuple of (sum, carry).
        fn half_adder(a: i8, b: i8) -> (i8, i8) {
            let (a, b) = (a.max(b), a.min(b));
            match (a, b) {
                (2, 2) => (-1, 1),
                (2, 1) => (-2, 1),
                (2, 0) => (2, 0),
                (2, -1) => (1, 0),
                (2, -2) => (0, 0),
                (1, 1) => (2, 0),
                (1, 0) => (1, 0),
                (1, -1) => (0, 0),
                (1, -2) => (-1, 0),
                (0, 0) => (0, 0),
                (0, -1) => (-1, 0),
                (0, -2) => (-2, 0),
                (-1, -1) => (-2, 0),
                (-1, -2) => (2, -1),
                (-2, -2) => (1, -1),
                _ => panic!("Unknown digit in {}, {}", a, b),
            }
        }

        /// Adds three SNAFU digits.
        /// Returns a tuple of (sum, carry).
        fn full_adder(a: i8, b: i8, carry_in: i8) -> (i8, i8) {
            let (sum, carry_temp1) = half_adder(a, b);
            let (sum, carry_temp2) = half_adder(sum, carry_in);

            assert!([-1, 0, 1].contains(&carry_temp1));
            assert!([-1, 0, 1].contains(&carry_temp2));

            let (carry, zero) = half_adder(carry_temp1, carry_temp2);
            assert_eq!(zero, 0);

            (sum, carry)
        }

        let mut carry = 0;
        for element in self.digits.iter_mut().zip_longest(rhs.digits.iter()) {
            match element {
                EitherOrBoth::Both(left, right) => {
                    let (result, carry_out) = full_adder(*left, *right, carry);
                    *left = result;
                    carry = carry_out;
                }
                EitherOrBoth::Left(left) => {
                    if carry == 0 {
                        break;
                    }

                    let (result, carry_out) = full_adder(*left, 0, carry);
                    *left = result;
                    carry = carry_out;
                }
                EitherOrBoth::Right(_) => unreachable!(),
            }
        }

        if carry != 0 {
            self.digits.push(carry);
        }
    }
}

impl AddAssign for SnafuInt {
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(&rhs);
    }
}

impl Add for SnafuInt {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.add_assign(&rhs);
        self
    }
}

impl Add<&SnafuInt> for SnafuInt {
    type Output = Self;

    fn add(mut self, rhs: &SnafuInt) -> Self::Output {
        self.add_assign(rhs);
        self
    }
}

impl Add for &SnafuInt {
    type Output = SnafuInt;

    fn add(self, rhs: Self) -> Self::Output {
        self.clone().add(rhs)
    }
}

impl Display for SnafuInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &digit in self.digits.iter().rev() {
            let digit = match digit {
                -2 => '=',
                -1 => '-',
                0 => '0',
                1 => '1',
                2 => '2',
                _ => panic!("Unexpected digit {}", digit),
            };
            write!(f, "{}", digit)?;
        }
        Ok(())
    }
}

impl FromStr for SnafuInt {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits: Result<Vec<i8>> = s
            .trim()
            .chars()
            .rev()
            .map(|digit| match digit {
                '=' => Ok(-2),
                '-' => Ok(-1),
                '0' => Ok(0),
                '1' => Ok(1),
                '2' => Ok(2),
                _ => bail!("Invalid digit {}", digit),
            })
            .collect();
        let digits = digits?;
        Ok(Self { digits })
    }
}

impl TryFrom<&SnafuInt> for i64 {
    type Error = anyhow::Error;

    fn try_from(snafu: &SnafuInt) -> Result<Self, Self::Error> {
        let mut result: i64 = 0;
        for (power, &digit) in snafu.digits.iter().enumerate() {
            let digit: i64 = digit.into();
            let value = digit
                .checked_mul(
                    5i64.checked_pow(power.try_into()?)
                        .context("pow overflow")?,
                )
                .context("Multiplication overflow")?;
            result = result.checked_add(value).context("Addition overflow")?;
        }
        Ok(result)
    }
}

impl TryFrom<SnafuInt> for i64 {
    type Error = anyhow::Error;

    fn try_from(value: SnafuInt) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

fn parse_input() -> Result<Vec<SnafuInt>> {
    input_lines()?
        .into_iter()
        .map(|line| line.parse())
        .collect()
}

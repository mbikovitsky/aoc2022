use std::collections::HashSet;

use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::iproduct;
use ndarray::{Array2, ArrayView2};

fn main() -> Result<()> {
    let forest = parse_input()?;

    let visible = count_visible(forest.view());
    dbg!(visible);

    let highest_score = highest_scenic_score(forest.view());
    dbg!(highest_score);

    Ok(())
}

fn count_visible(forest: ArrayView2<u8>) -> usize {
    let mut visible = HashSet::new();

    // Rows
    for (row, row_data) in forest.rows().into_iter().enumerate() {
        visible.extend(find_visible(row_data.iter().copied()).map(|column| (row, column)));
    }

    // Rows, reversed
    for (row, row_data) in forest.rows().into_iter().enumerate() {
        visible.extend(
            find_visible(row_data.iter().rev().copied())
                .map(|column| (row, row_data.len() - 1 - column)),
        );
    }

    // Columns
    for (column, column_data) in forest.columns().into_iter().enumerate() {
        visible.extend(find_visible(column_data.iter().copied()).map(|row| (row, column)));
    }

    // Columns, reversed
    for (column, column_data) in forest.columns().into_iter().enumerate() {
        visible.extend(
            find_visible(column_data.iter().rev().copied())
                .map(|row| (column_data.len() - 1 - row, column)),
        );
    }

    visible.len()
}

fn find_visible(heights: impl Iterator<Item = u8>) -> impl Iterator<Item = usize> {
    let mut max = -1i32;
    heights.enumerate().filter_map(move |(index, height)| {
        if i32::from(height) > max {
            max = i32::from(height);
            Some(index)
        } else {
            None
        }
    })
}

fn highest_scenic_score(forest: ArrayView2<u8>) -> usize {
    iproduct!(0..forest.nrows(), 0..forest.ncols())
        .map(|(row, col)| scenic_score(forest, (row, col)))
        .max()
        .unwrap()
}

fn scenic_score(forest: ArrayView2<u8>, tree: (usize, usize)) -> usize {
    // Left
    let mut left = 0;
    for column in (0..tree.1).rev() {
        left += 1;
        if forest[(tree.0, column)] >= forest[tree] {
            break;
        }
    }

    // Right
    let mut right = 0;
    for column in tree.1 + 1..forest.ncols() {
        right += 1;
        if forest[(tree.0, column)] >= forest[tree] {
            break;
        }
    }

    // Up
    let mut up = 0;
    for row in (0..tree.0).rev() {
        up += 1;
        if forest[(row, tree.1)] >= forest[tree] {
            break;
        }
    }

    // Down
    let mut down = 0;
    for row in tree.0 + 1..forest.nrows() {
        down += 1;
        if forest[(row, tree.1)] >= forest[tree] {
            break;
        }
    }

    left * right * up * down
}

fn parse_input() -> Result<Array2<u8>> {
    let lines = input_lines()?;

    let mut result = Array2::zeros((lines.len(), lines[0].len()));

    for (row, row_data) in lines.into_iter().enumerate() {
        if !row_data.is_ascii() {
            bail!("Expected ASCII string, got {}", row_data);
        }

        for (column, element) in row_data.into_bytes().into_iter().enumerate() {
            if !(b'0'..=b'9').contains(&element) {
                bail!("Expected character between '0' and '9', got {}", element);
            }

            result[(row, column)] = element - b'0';
        }
    }

    Ok(result)
}

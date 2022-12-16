use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader},
    ops::{Add, Sub},
    path::PathBuf,
};

use anyhow::{Context, Result};
use nalgebra::{Point2, Scalar};

const INPUTS_DIRECTORY: &str = "inputs";

pub fn input_file() -> Result<File> {
    let input_filename = match env::args_os().nth(1) {
        Some(filename) => filename,
        None => {
            let mut path: PathBuf = [
                OsStr::new(INPUTS_DIRECTORY),
                env::current_exe()
                    .context("Couldn't get executable filename")?
                    .file_stem()
                    .context("No executable filename")?,
            ]
            .iter()
            .collect();

            path.set_extension("txt");

            path.into_os_string()
        }
    };

    Ok(File::open(input_filename)?)
}

pub fn input_lines() -> Result<Vec<String>> {
    let reader = BufReader::new(input_file()?);

    let lines = reader.lines();

    let lines: std::result::Result<Vec<String>, _> = lines.collect();

    let lines = lines?;

    Ok(lines)
}

pub fn manhattan_distance<T>(a: &Point2<T>, b: &Point2<T>) -> T
where
    T: Scalar + Copy + Ord + Sub<Output = T> + Add<Output = T>,
{
    let delta_x = if a.x >= b.x { a.x - b.x } else { b.x - a.x };
    let delta_y = if a.y >= b.y { a.y - b.y } else { b.y - a.y };
    delta_x + delta_y
}

use anyhow::{bail, Context, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;

fn main() -> Result<()> {
    let datastream = parse_input()?;

    let sop = find_start_of_packet(&datastream).context("SOP not found")?;
    dbg!(sop);

    let som = find_start_of_message(&datastream).context("SOM not found")?;
    dbg!(som);

    Ok(())
}

/// Returns the index of the start-of-packet marker
fn find_start_of_packet(datastream: &[u8]) -> Option<usize> {
    datastream
        .windows(4)
        .find_position(|window| window.iter().all_unique())
        .map(|(index, _window)| index + 4)
}

fn find_start_of_message(datastream: &[u8]) -> Option<usize> {
    datastream
        .windows(14)
        .find_position(|window| window.iter().all_unique())
        .map(|(index, _window)| index + 14)
}

fn parse_input() -> Result<Vec<u8>> {
    let datastream = input_lines()?
        .into_iter()
        .exactly_one()
        .context("Expected only one input line")?;
    if !datastream.is_ascii() {
        bail!("Expected ASCII string");
    }
    Ok(datastream.into_bytes())
}

use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::iproduct;
use nalgebra::Point3;
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map_res},
    sequence::tuple,
    IResult,
};

fn main() -> Result<()> {
    let points = parse_input()?;

    let approximate_surface_area = approximate_surface_area(&points);
    dbg!(approximate_surface_area);

    Ok(())
}

fn approximate_surface_area(points: &[Point3<u32>]) -> u32 {
    let touching_faces = iproduct!(points, points)
        .filter(|(&a, &b)| are_points_adjacent(a, b))
        .count();
    let touching_faces: u32 = touching_faces.try_into().unwrap();

    let total_faces = points.len() * 6;
    let total_faces: u32 = total_faces.try_into().unwrap();

    total_faces - touching_faces
}

fn are_points_adjacent(a: Point3<u32>, b: Point3<u32>) -> bool {
    if a.xy() == b.xy() {
        a.z.abs_diff(b.z) == 1
    } else if a.xz() == b.xz() {
        a.y.abs_diff(b.y) == 1
    } else if a.yz() == b.yz() {
        a.x.abs_diff(b.x) == 1
    } else {
        false
    }
}

fn parse_input() -> Result<Vec<Point3<u32>>> {
    fn parse_int(input: &str) -> IResult<&str, u32> {
        map_res(digit1, str::parse)(input)
    }

    fn parse_point(input: &str) -> IResult<&str, Point3<u32>> {
        let parse_coordinates = tuple((parse_int, tag(","), parse_int, tag(","), parse_int));
        let point_from_coordinates =
            |(x, _, y, _, z)| -> Result<Point3<u32>> { Ok(Point3::new(x, y, z)) };
        map_res(parse_coordinates, point_from_coordinates)(input)
    }

    input_lines()?
        .into_iter()
        .map(|line| match all_consuming(parse_point)(&line) {
            Ok((_, point)) => Ok(point),
            Err(error) => bail!("Invalid coordinates: {}", error),
        })
        .collect()
}

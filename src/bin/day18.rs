use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::iproduct;
use nalgebra::Point3;
use ndarray::{Array3, ArrayView3};
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

    let surface_area = compute_surface_area(&points);
    dbg!(surface_area);

    Ok(())
}

fn approximate_surface_area(points: &[Point3<u8>]) -> u32 {
    let touching_faces = iproduct!(points, points)
        .filter(|(&a, &b)| are_points_adjacent(a, b))
        .count();
    let touching_faces: u32 = touching_faces.try_into().unwrap();

    let total_faces = points.len() * 6;
    let total_faces: u32 = total_faces.try_into().unwrap();

    total_faces - touching_faces
}

fn are_points_adjacent(a: Point3<u8>, b: Point3<u8>) -> bool {
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

fn compute_surface_area(points: &[Point3<u8>]) -> u32 {
    let map = {
        // Compute the shape of the map, with some room to spare.
        // We need enough space to store all the points, and a buffer plane
        // of width 1 on each side of the shape.
        // The buffer is there so that we have the shape surrounded by empty space,
        // which means every outside face is reachable from any empty point
        // on the outside.
        let max_x: usize = points.iter().map(|point| point.x).max().unwrap().into();
        let max_y: usize = points.iter().map(|point| point.y).max().unwrap().into();
        let max_z: usize = points.iter().map(|point| point.z).max().unwrap().into();
        let shape = (max_x + 3, max_y + 3, max_z + 3);

        let mut map = Array3::zeros(shape);

        for point in points {
            let x: usize = point.x.into();
            let y: usize = point.y.into();
            let z: usize = point.z.into();
            map[(x + 1, y + 1, z + 1)] = 1u8;
        }

        map
    };

    let mut to_visit = vec![Point3::new(0u8, 0, 0)];

    let mut discovered = Array3::zeros(map.raw_dim());

    let mut surface_area = 0;

    while let Some(point) = to_visit.pop() {
        if std::mem::replace(
            &mut discovered[(point.x.into(), point.y.into(), point.z.into())],
            1u8,
        ) == 1
        {
            continue;
        }

        for neighbour in adjacent_points(point, map.view()) {
            let neighbour_coords = (neighbour.x.into(), neighbour.y.into(), neighbour.z.into());

            // Skip already visited points in space
            if discovered[neighbour_coords] == 1 {
                continue;
            }

            // If this spot is occupied, then we have a face to add to the surface area
            if map[neighbour_coords] == 1 {
                surface_area += 1;
                continue;
            }

            to_visit.push(neighbour);
        }
    }

    surface_area
}

fn adjacent_points(point: Point3<u8>, map: ArrayView3<u8>) -> impl Iterator<Item = Point3<u8>> {
    [
        if point.x > 0 {
            Some(Point3::new(point.x - 1, point.y, point.z))
        } else {
            None
        },
        if usize::from(point.x) + 1 < map.shape()[0] {
            Some(Point3::new(point.x + 1, point.y, point.z))
        } else {
            None
        },
        if point.y > 0 {
            Some(Point3::new(point.x, point.y - 1, point.z))
        } else {
            None
        },
        if usize::from(point.y) + 1 < map.shape()[1] {
            Some(Point3::new(point.x, point.y + 1, point.z))
        } else {
            None
        },
        if point.z > 0 {
            Some(Point3::new(point.x, point.y, point.z - 1))
        } else {
            None
        },
        if usize::from(point.z) + 1 < map.shape()[2] {
            Some(Point3::new(point.x, point.y, point.z + 1))
        } else {
            None
        },
    ]
    .into_iter()
    .flatten()
}

fn parse_input() -> Result<Vec<Point3<u8>>> {
    fn parse_int(input: &str) -> IResult<&str, u8> {
        map_res(digit1, str::parse)(input)
    }

    fn parse_point(input: &str) -> IResult<&str, Point3<u8>> {
        let parse_coordinates = tuple((parse_int, tag(","), parse_int, tag(","), parse_int));
        let point_from_coordinates =
            |(x, _, y, _, z)| -> Result<Point3<u8>> { Ok(Point3::new(x, y, z)) };
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

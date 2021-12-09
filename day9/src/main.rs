use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::fs;
use std::str::FromStr;

struct Heightmap {
    data: Vec<Vec<usize>>,
    max_x: usize,
    max_y: usize,
}

impl Heightmap {
    fn new(data: Vec<Vec<usize>>) -> Result<Self> {
        if data.len() == 0 {
            return Err(anyhow!("heightmap cannot be empty"));
        }

        let max_x = data[0].len();
        let max_y = data.len();

        Ok(Self { data, max_x, max_y })
    }

    fn risk_level(&self) -> usize {
        self.low_points()
            .into_iter()
            .map(|(x, y)| self.data[y][x] + 1)
            .sum()
    }

    fn neighbours(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        let xi = x as isize;
        let yi = y as isize;

        let max_x = self.max_x;
        let max_y = self.max_y;

        let point_candidates = [(xi - 1, yi), (xi + 1, yi), (xi, yi - 1), (xi, yi + 1)];
        point_candidates
            .into_iter()
            .filter(move |(x, y)| {
                *x >= 0 && (*x as usize) < max_x && *y >= 0 && (*y as usize) < max_y
            })
            .map(|(x, y)| (x as usize, y as usize))
    }

    fn low_points(&self) -> Vec<(usize, usize)> {
        (0..self.max_x)
            .cartesian_product(0..self.max_y)
            .filter(|(x, y)| {
                self.neighbours(*x, *y)
                    .all(|(nx, ny)| self.data[ny][nx] > self.data[*y][*x])
            })
            .collect()
    }

    fn basins(&self) -> Vec<(usize, usize)> {
        let mut basin_map = vec![vec![0; self.max_x]; self.max_y];
        let mut basins = vec![];

        let mut basin_idx = 1;
        for (x, y) in self.low_points() {
            let mut stack = vec![(x, y)];
            basins.push((basin_idx, 0));
            let (_, basin_size) = basins.last_mut().unwrap();
            basin_map[y][x] = basin_idx;

            while let Some((x, y)) = stack.pop() {
                let value = self.data[y][x];
                *basin_size += 1;

                for (nx, ny) in self.neighbours(x, y) {
                    let not_basin_already = basin_map[ny][nx] == 0;
                    let forms_basin = self.data[ny][nx] != 9 && self.data[ny][nx] > value;

                    if not_basin_already && forms_basin {
                        basin_map[ny][nx] = basin_idx;
                        stack.push((nx, ny));
                    }
                }
            }

            basin_idx += 1;
        }

        basins
    }
}

fn digit_to_usize(digit: char) -> Result<usize> {
    match digit {
        '0' => Ok(0),
        '1' => Ok(1),
        '2' => Ok(2),
        '3' => Ok(3),
        '4' => Ok(4),
        '5' => Ok(5),
        '6' => Ok(6),
        '7' => Ok(7),
        '8' => Ok(8),
        '9' => Ok(9),
        _ => Err(anyhow!("character is not a digit: {}", digit)),
    }
}

impl FromStr for Heightmap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = s
            .lines()
            .map(str::chars)
            .map(|chars| chars.map(digit_to_usize).collect::<Result<Vec<_>>>())
            .collect::<Result<Vec<_>>>()?;

        Self::new(data)
    }
}

fn main() -> Result<()> {
    let heightmap: Heightmap = fs::read_to_string("./input")?.parse()?;

    println!(
        "Total risk level of a heightmap: {}",
        heightmap.risk_level()
    );

    let mut basins = heightmap.basins();
    basins.sort_unstable_by_key(|basin| basin.1);

    let three_largest_basins_size_product: usize =
        basins.iter().rev().take(3).map(|basin| basin.1).product();

    println!(
        "Product of three largest basins' size: {}",
        three_largest_basins_size_product
    );

    Ok(())
}

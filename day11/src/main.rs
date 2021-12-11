use itertools::Itertools;
use std::{collections::HashSet, convert::Infallible, fmt::Display, fs, str::FromStr};

const GRID_SIZE: usize = 10;

#[derive(Debug)]
struct OctopusGrid {
    board: [[u8; GRID_SIZE]; GRID_SIZE],
}

impl FromStr for OctopusGrid {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board = [[0; GRID_SIZE]; GRID_SIZE];

        s.lines().take(GRID_SIZE).enumerate().for_each(|(y, line)| {
            line.chars()
                .take(GRID_SIZE)
                .enumerate()
                .for_each(|(x, digit)| board[y][x] = (digit as i32 - 0x30) as u8)
        });

        Ok(Self { board })
    }
}

impl Display for OctopusGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.board.iter() {
            for o in row.iter() {
                write!(f, "{}", o)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl OctopusGrid {
    fn step(&mut self) -> usize {
        let mut step_flashes = 0;

        for row in self.board.iter_mut() {
            for octopus in row.iter_mut() {
                *octopus += 1;
            }
        }

        let mut flash_positions = (0..GRID_SIZE)
            .cartesian_product(0..GRID_SIZE)
            .filter(|(x, y)| self.board[*y][*x] >= 10)
            .collect::<Vec<_>>();

        let mut already_flashed = HashSet::with_capacity(100);
        already_flashed.extend(flash_positions.iter().copied());

        while let Some((x, y)) = flash_positions.pop() {
            step_flashes += 1;

            Self::neighbours(x, y).for_each(|(nx, ny)| {
                self.board[ny][nx] += 1;
                if self.board[ny][nx] >= 10 && !already_flashed.contains(&(nx, ny)) {
                    flash_positions.push((nx, ny));
                    already_flashed.insert((nx, ny));
                }
            })
        }

        already_flashed
            .into_iter()
            .for_each(|(x, y)| self.board[y][x] = 0);

        step_flashes
    }

    fn synchronized_step(&mut self) -> usize {
        std::iter::repeat(())
            .take_while(|_| self.step() != GRID_SIZE * GRID_SIZE)
            .count()
            + 1
    }

    fn neighbours(x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        [-1, 0, 1]
            .into_iter()
            .cartesian_product([-1, 0, 1].into_iter())
            .map(move |(i, j)| (x as isize + i, y as isize + j))
            .filter(move |(nx, ny)| {
                *nx > -1
                    && *ny > -1
                    && *nx < GRID_SIZE as isize
                    && *ny < GRID_SIZE as isize
                    && (*nx, *ny) != (x as isize, y as isize)
            })
            .map(|(nx, ny)| (nx as usize, ny as usize))
    }
}

fn main() -> anyhow::Result<()> {
    let mut cave: OctopusGrid = fs::read_to_string("./input")?.parse()?;

    let steps = (0..100).map(|_| cave.step());

    println!(
        "Number of flashes after 100 seconds: {}",
        steps.sum::<usize>()
    );

    let mut cave: OctopusGrid = fs::read_to_string("./input")?.parse()?;

    println!(
        "Octopuses synchronize in {} steps",
        cave.synchronized_step()
    );

    Ok(())
}

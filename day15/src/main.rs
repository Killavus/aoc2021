use std::collections::BinaryHeap;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::{convert::Infallible, str::FromStr};

use fxhash::FxHashSet;

struct CaveMap {
    data: Vec<Vec<usize>>,
    max_x: usize,
    max_y: usize,
}

#[derive(Debug)]
struct Point(usize, usize, usize);

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0).map(|cmp| cmp.reverse())
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}
impl Eq for Point {}

impl Display for CaveMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.iter() {
            for col in row.iter() {
                write!(f, "{}", col)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl FromStr for CaveMap {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cave_map;
        let max_x;
        let max_y = s.lines().count();

        if let Some(first_line) = s.lines().take(1).next() {
            max_x = first_line.len();
            cave_map = vec![vec![10; max_x]; max_y];
        } else {
            panic!("Malformed input: empty cave map");
        }

        for (y, row) in s.lines().enumerate() {
            for (x, risk_level) in row.chars().enumerate() {
                cave_map[y][x] = (risk_level as u8 - '0' as u8) as usize;
            }
        }

        Ok(Self {
            data: cave_map,
            max_x,
            max_y,
        })
    }
}

enum CaveType {
    PartialCave,
    FullCave,
}

impl CaveMap {
    fn neighbours(
        &self,
        x: usize,
        y: usize,
        cave_type: &CaveType,
    ) -> impl Iterator<Item = (usize, usize)> {
        let xi = x as isize;
        let yi = y as isize;

        let max_x = self.max_x;
        let max_y = self.max_y;

        let multiplier = match &cave_type {
            CaveType::FullCave => 5,
            CaveType::PartialCave => 1,
        };

        let point_candidates = [(xi - 1, yi), (xi + 1, yi), (xi, yi - 1), (xi, yi + 1)];
        point_candidates
            .into_iter()
            .filter(move |(x, y)| {
                *x >= 0
                    && (*x as usize) < (max_x * multiplier)
                    && *y >= 0
                    && (*y as usize) < (max_y * multiplier)
            })
            .map(|(x, y)| (x as usize, y as usize))
    }

    fn cost_of(&self, x: usize, y: usize) -> usize {
        if x < self.max_x && y < self.max_y {
            self.data[y][x]
        } else {
            let add_x = x / self.max_x;
            let add_y = y / self.max_y;
            let raw_cost = self.data[y % self.max_y][x % self.max_y];

            if (raw_cost + add_x + add_y) > 9 {
                raw_cost + add_x + add_y - 9
            } else {
                raw_cost + add_x + add_y
            }
        }
    }

    fn lowest_risk_level(&self, cave_type: CaveType) -> usize {
        let mut cost_heap = BinaryHeap::new();
        cost_heap.push(Point(0, 0, 0));
        let mut used = FxHashSet::default();
        used.insert((0, 0));

        let multiplier = match cave_type {
            CaveType::FullCave => 5,
            CaveType::PartialCave => 1,
        };

        while let Some(Point(cost, x, y)) = cost_heap.pop() {
            if (x, y) == (self.max_x * multiplier - 1, self.max_y * multiplier - 1) {
                return cost;
            }

            self.neighbours(x, y, &cave_type).for_each(|(nx, ny)| {
                if !used.contains(&(nx, ny)) {
                    cost_heap.push(Point(cost + self.cost_of(nx, ny), nx, ny));
                    used.insert((nx, ny));
                }
            });
        }

        usize::MAX
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cave_map: CaveMap = fs::read_to_string("./input")?.parse()?;

    println!(
        "Lowest risk level achievable in partial cave while traversing is {}",
        cave_map.lowest_risk_level(CaveType::PartialCave)
    );

    println!(
        "Lowest risk level achievable in full cave while traversing is {}",
        cave_map.lowest_risk_level(CaveType::FullCave)
    );

    Ok(())
}

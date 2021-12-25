use std::error::Error;
use std::fs;
use std::{collections::HashSet, convert::Infallible, str::FromStr};

#[derive(Clone, Debug)]
struct CucumberMap {
    east_cucumbers: HashSet<(usize, usize)>,
    south_cucumbers: HashSet<(usize, usize)>,
    boundaries: (usize, usize),
}

impl CucumberMap {
    fn next_east(&self, pos: &(usize, usize)) -> (usize, usize) {
        ((pos.0 + 1) % self.boundaries.0, pos.1)
    }

    fn next_south(&self, pos: &(usize, usize)) -> (usize, usize) {
        (pos.0, (pos.1 + 1) % self.boundaries.1)
    }

    fn occupied(&self, pos: &(usize, usize)) -> bool {
        self.east_cucumbers.contains(pos) || self.south_cucumbers.contains(pos)
    }

    fn perform_step<F>(
        &self,
        cucumbers: &HashSet<(usize, usize)>,
        step_fn: F,
    ) -> (usize, HashSet<(usize, usize)>)
    where
        F: Fn(&(usize, usize)) -> (usize, usize),
    {
        let mut moves = 0;
        let mut new_cucumbers = HashSet::new();
        for cucumber in cucumbers.iter().copied() {
            let next_pos = step_fn(&cucumber);
            if !self.occupied(&next_pos) {
                new_cucumbers.insert(next_pos);
                moves += 1;
            } else {
                new_cucumbers.insert(cucumber);
            }
        }

        (moves, new_cucumbers)
    }

    fn step(&mut self) -> usize {
        let mut moves = 0;
        let (east_moves, east_cucumbers) =
            self.perform_step(&self.east_cucumbers, |pos| self.next_east(pos));
        self.east_cucumbers = east_cucumbers;
        moves += east_moves;

        let (south_moves, south_cucumbers) =
            self.perform_step(&self.south_cucumbers, |pos| self.next_south(pos));

        self.south_cucumbers = south_cucumbers;
        moves += south_moves;

        moves
    }
}

impl FromStr for CucumberMap {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let max_y = s.lines().count();
        let max_x = s.lines().next().expect("cucumber map is empty").len();
        let mut west_cucumbers = HashSet::new();
        let mut south_cucumbers = HashSet::new();

        let boundaries = (max_x, max_y);

        for (y, line) in s.lines().enumerate() {
            for (x, field) in line.chars().enumerate() {
                match field {
                    '>' => {
                        west_cucumbers.insert((x, y));
                    }
                    'v' => {
                        south_cucumbers.insert((x, y));
                    }
                    _ => {}
                }
            }
        }

        Ok(Self {
            boundaries,
            east_cucumbers: west_cucumbers,
            south_cucumbers,
        })
    }
}

fn steps_to_stop(cucumber_map: &CucumberMap) -> usize {
    let mut cucumber_map = cucumber_map.clone();
    let mut steps = 0;

    loop {
        steps += 1;
        if cucumber_map.step() == 0 {
            break;
        }
    }

    steps
}

fn main() -> Result<(), Box<dyn Error>> {
    let cucumber_map: CucumberMap = fs::read_to_string("./input")?.parse()?;

    println!(
        "Sea cucumbers stop moving after {} steps",
        steps_to_stop(&cucumber_map)
    );

    Ok(())
}

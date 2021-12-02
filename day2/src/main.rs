use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
struct DirectionInvalidFormat;

impl Display for DirectionInvalidFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} - failed to parse direction format", self)
    }
}

impl Error for DirectionInvalidFormat {}

#[derive(Debug)]
enum Direction {
    Up(usize),
    Down(usize),
    Forward(usize),
}

impl FromStr for Direction {
    type Err = DirectionInvalidFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splitted = s.split_ascii_whitespace();
        let command = splitted.next().ok_or(DirectionInvalidFormat)?;
        let units = splitted
            .next()
            .ok_or(DirectionInvalidFormat)?
            .parse::<usize>()
            .map_err(|_| DirectionInvalidFormat)?;

        match command {
            "forward" => Ok(Direction::Forward(units)),
            "up" => Ok(Direction::Up(units)),
            "down" => Ok(Direction::Down(units)),
            _ => Err(DirectionInvalidFormat),
        }
    }
}

impl Direction {
    fn process(&self, (horizontal, depth): (usize, usize)) -> (usize, usize) {
        match self {
            Self::Forward(u) => (horizontal + u, depth),
            Self::Up(u) => (horizontal, depth - u),
            Self::Down(u) => (horizontal, depth + u),
        }
    }

    fn process_aimed(
        &self,
        (horizontal, depth, aim): (usize, usize, usize),
    ) -> (usize, usize, usize) {
        match self {
            Self::Forward(u) => (horizontal + u, depth + aim * u, aim),
            Self::Up(u) => (horizontal, depth, aim - u),
            Self::Down(u) => (horizontal, depth, aim + u),
        }
    }
}

fn read_all(path: impl AsRef<Path>) -> Result<Vec<Direction>, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .flat_map(str::parse)
        .collect())
}

fn final_shuttle_position(directions: &[Direction]) -> (usize, usize) {
    directions
        .iter()
        .fold((0, 0), |total, direction| direction.process(total))
}

fn final_shuttle_position_aimed(directions: &[Direction]) -> (usize, usize) {
    let result = directions
        .iter()
        .fold((0, 0, 0), |total, direction| direction.process_aimed(total));

    (result.0, result.1)
}

fn main() -> Result<(), Box<dyn Error>> {
    let directions = read_all("./input")?;

    let final_pos = final_shuttle_position(&directions);
    let final_pos_aimed = final_shuttle_position_aimed(&directions);

    println!(
        "shuttle position {:?}, multiplied is {}",
        final_pos,
        final_pos.0 * final_pos.1
    );
    println!(
        "shuttle position with aim {:?}, multiplied is {}",
        final_pos_aimed,
        final_pos_aimed.0 * final_pos_aimed.1
    );
    Ok(())
}

use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub struct DirectionInvalidFormat;

impl Display for DirectionInvalidFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} - failed to parse direction format", self)
    }
}

impl Error for DirectionInvalidFormat {}

#[derive(Debug)]
pub enum Direction {
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
    pub fn process(&self, (horizontal, depth): (usize, usize)) -> (usize, usize) {
        match self {
            Self::Forward(u) => (horizontal + u, depth),
            Self::Up(u) => (horizontal, depth - u),
            Self::Down(u) => (horizontal, depth + u),
        }
    }

    pub fn process_aimed(
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

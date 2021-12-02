use std::error::Error;
use std::fs;
use std::path::Path;

mod direction;
use direction::Direction;

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

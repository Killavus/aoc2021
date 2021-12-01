use std::error::Error;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use utils::consecutive_pairs;

fn measure_increase(total: usize, (current, next): (usize, usize)) -> usize {
    if next > current {
        total + 1
    } else {
        total
    }
}

fn measurement_increases(sonar_data: &[usize]) -> usize {
    let point_pairs = consecutive_pairs(sonar_data.iter());

    point_pairs.fold(0, |total, (current, next)| {
        measure_increase(total, (*current, *next))
    })
}

fn measurement_window_increases(sonar_data: &[usize]) -> usize {
    let window_sums = consecutive_pairs(sonar_data.windows(3));

    window_sums.fold(0, |total, (window, next_window)| {
        measure_increase(total, (window.iter().sum(), next_window.iter().sum()))
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let sonar_data: Result<Vec<usize>, Box<dyn Error>> = BufReader::new(File::open("./input")?)
        .lines()
        .map(|line| {
            line.map_err(Into::into)
                .and_then(|text| text.parse::<usize>().map_err(Into::into))
        })
        .collect();
    let sonar_data = sonar_data?;

    println!("{}", measurement_increases(&sonar_data));
    println!("{}", measurement_window_increases(&sonar_data));
    Ok(())
}

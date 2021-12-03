use bitvec::field::BitField;
use bitvec::order::Msb0;
use std::error::Error;
use std::fs;
use std::io::{self};
use std::path::Path;

use bitvec::prelude::BitVec;

fn read_diagnostic_report(path: impl AsRef<Path>) -> Result<String, io::Error> {
    Ok(fs::read_to_string(path)?)
}

fn report_verticals(report: &str) -> Vec<BitVec<Msb0>> {
    let line_length = report
        .lines()
        .next()
        .expect("report should not be empty")
        .len();
    let line_count = report.lines().count();
    let mut verticals = vec![BitVec::<bitvec::order::Msb0>::with_capacity(line_count); line_length];

    for line in report.lines() {
        for (index, bit) in line.chars().enumerate() {
            verticals[index].push(bit == '1');
        }
    }

    verticals
}

fn power_consumption(report: &str) -> usize {
    let line_length = report
        .lines()
        .next()
        .expect("report should not be empty")
        .len();
    let verticals = report_verticals(&report);
    let mut gamma_rate: BitVec<Msb0> = BitVec::with_capacity(line_length);
    let mut epsilon_rate: BitVec<Msb0> = BitVec::with_capacity(line_length);

    for vertical in verticals.iter() {
        let vertical_bitslice = vertical.as_bitslice();
        let ones_count = vertical_bitslice.count_ones();
        let zeros_count = vertical_bitslice.count_zeros();

        gamma_rate.push(ones_count > zeros_count);
        epsilon_rate.push(zeros_count > ones_count);
    }

    let gamma_rate = gamma_rate.load::<usize>();
    let epsilon_rate = epsilon_rate.load::<usize>();

    gamma_rate * epsilon_rate
}

fn partition_report(
    horizontals: &mut [BitVec<Msb0>],
    verticals: &mut [BitVec<Msb0>],
    index: usize,
    most_common_bit: bool,
) -> usize {
    let mut next_idx = 0;
    for i in 0..horizontals.len() {
        if horizontals[i][index] == most_common_bit {
            horizontals.swap(next_idx, i);
            for j in 0..horizontals[i].len() {
                verticals[j].swap(next_idx, i);
            }
            next_idx += 1;
        }
    }

    next_idx
}

fn search_variable(
    verticals: &mut [BitVec<Msb0>],
    horizontals: &mut [BitVec<Msb0>],
    inverse: bool,
) -> usize {
    let mut search_set = horizontals.len();
    for i in 0..verticals[0].len() {
        if search_set == 1 {
            break;
        }

        let vertical = &verticals[i];
        let vertical_bitslice = &vertical[0..search_set];
        let ones_count = vertical_bitslice.count_ones();
        let zeros_count = vertical_bitslice.count_zeros();
        let condition = if inverse {
            ones_count < zeros_count
        } else {
            ones_count >= zeros_count
        };

        search_set = partition_report(&mut horizontals[0..search_set], verticals, i, condition);
    }

    horizontals[0].load::<usize>()
}

fn life_support_rating(report: &str) -> usize {
    let mut verticals = report_verticals(&report);
    let mut horizontals = report
        .lines()
        .map(|horizontal| {
            let mut bitvec: BitVec<Msb0> = BitVec::with_capacity(horizontal.len());
            horizontal.chars().for_each(|bit| bitvec.push(bit == '1'));
            bitvec
        })
        .collect::<Vec<_>>();

    let oxygen_generator_rating = search_variable(&mut verticals, &mut horizontals, false);
    let co2_scrubber_rating = search_variable(&mut verticals, &mut horizontals, true);

    oxygen_generator_rating * co2_scrubber_rating
}

fn main() -> Result<(), Box<dyn Error>> {
    let diagnostic_report = read_diagnostic_report("./input")?;

    println!(
        "power consumption = {}",
        power_consumption(&diagnostic_report)
    );

    println!(
        "life support rating = {}",
        life_support_rating(&diagnostic_report)
    );

    Ok(())
}

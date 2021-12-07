use std::error::Error;
use std::fs;

fn fuel_cost_for_move(
    crab_positions: &[isize],
    target_position: isize,
    cost_fn: &FuelCostFn,
) -> isize {
    crab_positions
        .iter()
        .copied()
        .map(|crab_position| cost_fn(crab_position, target_position))
        .sum()
}

fn optimal_crab_alignment(crab_positions: &[isize], cost_fn: &FuelCostFn) -> (usize, isize) {
    let max_x = crab_positions
        .iter()
        .copied()
        .max()
        .expect("crab positions should be non-empty");

    let position_costs: Vec<isize> = (0..=max_x)
        .map(|position| fuel_cost_for_move(&crab_positions, position, cost_fn))
        .collect();

    position_costs
        .into_iter()
        .enumerate()
        .min_by_key(|(_, fuel_cost)| *fuel_cost)
        .expect("crab positions should be non-empty")
}

type FuelCostFn = dyn Fn(isize, isize) -> isize;

fn linear_fuel_cost(crab_position: isize, target_position: isize) -> isize {
    (crab_position - target_position).abs()
}

fn increasing_fuel_cost(crab_position: isize, target_position: isize) -> isize {
    let linear_cost = linear_fuel_cost(crab_position, target_position);

    if linear_cost > 0 {
        // 1 + 2 + 3 + ... + n = [n(n+1)]/2
        ((1 + linear_cost) * linear_cost) / 2
    } else {
        0
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let crab_positions = fs::read_to_string("./input")?
        .lines()
        .flat_map(|line| line.split(','))
        .flat_map(str::parse)
        .collect::<Vec<isize>>();

    let (best_position, fuel_cost) = optimal_crab_alignment(&crab_positions, &linear_fuel_cost);

    println!(
        "Linear fuel cost: Best alignment at position {}, fuel cost: {}",
        best_position, fuel_cost
    );

    let (best_position, fuel_cost) = optimal_crab_alignment(&crab_positions, &increasing_fuel_cost);

    println!(
        "Increasing fuel cost: Best alignment at position {}, fuel cost: {}",
        best_position, fuel_cost
    );

    Ok(())
}

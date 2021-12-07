use rand::prelude::{IteratorRandom, SliceRandom};
use rand::Rng;
use std::error::Error;
use std::fs;

fn fuel_cost_for_move<F>(crab_positions: &[isize], target_position: isize, cost_fn: F) -> isize
where
    F: Fn(isize, isize) -> isize,
{
    crab_positions
        .iter()
        .copied()
        .map(|crab_position| cost_fn(crab_position, target_position))
        .sum()
}

/// This algorithm works for every cost function & positons x_0,x_1,...,x_n and
/// performs its task in O(m * n) where m = max(x_i), n = len(x_i).
fn optimal_crab_alignment_generic<F>(crab_positions: &[isize], cost_fn: F) -> (usize, isize)
where
    F: Fn(isize, isize) -> isize + Copy,
{
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

fn partition_pivot(slice: &mut [isize], pivot_idx: usize) -> usize {
    let pivot_val = slice[pivot_idx];
    slice.swap(pivot_idx, slice.len() - 1);

    let mut next_idx = 0;

    for idx in 0..(slice.len() - 1) {
        if slice[idx] <= pivot_val {
            slice.swap(idx, next_idx);
            next_idx += 1;
        }
    }

    slice.swap(next_idx, slice.len() - 1);
    next_idx
}

fn find_kth_element(collection: &[isize], mut k: usize) -> isize {
    let mut rng = rand::thread_rng();
    let mut copied = Vec::with_capacity(collection.len());
    copied.extend_from_slice(collection);

    let mut search_slice = copied.as_mut_slice();

    loop {
        if k == 0 {
            break;
        }

        if search_slice.len() == 1 {
            return search_slice[0];
        }

        let pivot_idx = (0..search_slice.len())
            .choose(&mut rng)
            .expect("need non-empty search slice");

        let pivot_kth = partition_pivot(&mut search_slice, pivot_idx);

        if pivot_kth > k {
            search_slice = &mut search_slice[0..pivot_kth];
        } else if pivot_kth == k {
            return search_slice[pivot_kth];
        } else {
            search_slice = &mut search_slice[pivot_kth..];
            k -= pivot_kth;
        }
    }

    search_slice[k]
}

/// This solution is assuming that L1 norm (d(x, y) = |x - y|) is used as crab distance function.
/// Using this assumption it can be shown that the optimal solution is the median of crab sequences.
/// L(y) = \sum_{i=0}^{n} |y - x_i|
/// Taking derivative over y we get: \frac{d}{dy}L(y) = \sum_{i=0}^{n} \sgn{y - x_i}.
/// Taking a median means that equal amount of elements are sgn -1 and 1 so it minimizes loss function.
/// Taking that into consideration optimal alignment can be found in O(n).
fn optimal_crab_alignment_l1(crab_positions: &[isize]) -> (isize, isize) {
    let median;
    if crab_positions.len() % 2 == 1 {
        median = find_kth_element(&crab_positions, crab_positions.len() / 2);
    } else {
        let (m_1, m_2) = (
            find_kth_element(&crab_positions, crab_positions.len() / 2),
            find_kth_element(&crab_positions, (crab_positions.len() / 2) - 1),
        );

        median = (m_1 + m_2) / 2;
    }

    (
        median,
        crab_positions
            .iter()
            .copied()
            .map(|crab_position| linear_fuel_cost(crab_position, median))
            .sum(),
    )
}

/// This solution is assuming that fuel cost is given by gauss sum from 1 to |target_position - crab_position|.
/// With applying the similar optimalisation logic like in L1 case it can be shown that optimal lies +/- 1/2 of
/// average of crab positions.
/// This solution uses this fact to calculate the result in O(n).
fn optimal_crab_alignment_gauss_sum(crab_positions: &[isize]) -> (isize, isize) {
    let crab_positions_sum = crab_positions.iter().copied().sum::<isize>();
    let crabs_total = crab_positions.len();

    let average = (crab_positions_sum as f64) / (crabs_total as f64);

    let candidate_floor = average.floor() as isize;
    let candidate_ceil = average.ceil() as isize;

    let result_floor = crab_positions
        .iter()
        .copied()
        .map(|crab_position| increasing_fuel_cost(crab_position, candidate_floor))
        .sum();
    let result_ceil = crab_positions
        .iter()
        .copied()
        .map(|crab_position| increasing_fuel_cost(crab_position, candidate_ceil))
        .sum();

    if result_floor < result_ceil {
        (candidate_floor, result_floor)
    } else {
        (candidate_ceil, result_ceil)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let crab_positions = fs::read_to_string("./input")?
        .lines()
        .flat_map(|line| line.split(','))
        .flat_map(str::parse)
        .collect::<Vec<isize>>();

    let (best_position, fuel_cost) = optimal_crab_alignment_l1(&crab_positions);

    println!(
        "Linear fuel cost: Best alignment at position {}, fuel cost: {}",
        best_position, fuel_cost
    );

    let (best_position, fuel_cost) = optimal_crab_alignment_gauss_sum(&crab_positions);

    println!(
        "Increasing fuel cost: Best alignment at position {}, fuel cost: {}",
        best_position, fuel_cost
    );

    Ok(())
}

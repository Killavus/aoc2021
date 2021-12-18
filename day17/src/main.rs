use anyhow::{anyhow, Result};
use std::fs;
use std::str::FromStr;

struct TargetArea {
    x: (isize, isize),
    y: (isize, isize),
}

fn solve_bounds(start_v: isize, limit: isize) -> Option<(f64, f64)> {
    let b = 2 * start_v + 1;
    let b_sq = b * b;
    let c = -8 * limit;

    if b_sq + c < 0 {
        return None;
    }

    let b_f = b as f64;
    let c_f = c as f64;
    let b_sq_f = b_sq as f64;
    let delta_sq = b_sq_f + c_f;

    let n1 = (-b_f + delta_sq.sqrt()) / -2.0;
    let n2 = (-b_f - delta_sq.sqrt()) / -2.0;

    Some((n1, n2))
}

impl FromStr for TargetArea {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let without_prefix = s
            .trim_end()
            .strip_prefix("target area: ")
            .ok_or(anyhow!("failed to find data prefix: {}", s))?;

        let mut axes = without_prefix.split(", ");

        let x_axis = axes
            .next()
            .ok_or(anyhow!("failed to find x-axis data: {}", s))?;

        let y_axis = axes
            .next()
            .ok_or(anyhow!("failed to find y-axis data: {}", s))?;

        let mut x_data = x_axis
            .strip_prefix("x=")
            .map(|data| data.split(".."))
            .ok_or(anyhow!("failed to isolate x-axis data: {}", s))?;

        let (xs, xe) = (
            x_data
                .next()
                .ok_or(anyhow!("failed to find x-axis start: {}", s))?
                .parse()?,
            x_data
                .next()
                .ok_or(anyhow!("failed to find x-axis end: {}", s))?
                .parse()?,
        );

        let mut y_data = y_axis
            .strip_prefix("y=")
            .map(|data| data.split(".."))
            .ok_or(anyhow!("failed to isolate y-axis data: {}", s))?;

        let (ys, ye) = (
            y_data
                .next()
                .ok_or(anyhow!("failed to find y-axis start: {}", s))?
                .parse()?,
            y_data
                .next()
                .ok_or(anyhow!("failed to find y-axis end: {}", s))?
                .parse()?,
        );

        Ok(Self {
            x: (xs, xe),
            y: (ys, ye),
        })
    }
}

fn main() -> Result<()> {
    let area: TargetArea = fs::read_to_string("./input")?.parse()?;

    // For every positive v it achieves it's peak after y steps. This is because it is when y starts to go into negative.
    // The value for it's apex is (y^2 + y) / 2 which is from closed form of distance function: [ny + (n - 1) * n] / 2.
    // You can substitute n by y and you get this result.
    //
    // Open question: Can you set up X range in a way that every possible x is moving through it and not stopping at it?
    // I assume it is impossible to get this result.
    let y_end = area.y.0;
    let max_y = -(y_end + 1);
    let max_h = (max_y * max_y + max_y) / 2;

    println!("Maximum style points achieved at height {}", max_h);

    let mut distinct = 0;
    let mut x_solutions = vec![];
    for x in 1..=area.x.1 {
        let after_xs = solve_bounds(x, area.x.0);

        if let Some((min_n, max_n)) = after_xs {
            // Parabola roots of distance function nx * (n(n-1))/2 - x_s = x(n) - x_s.
            // You need to find steps making parabola cross x_s, so:
            // x(n) >= x_s
            // This parabola has negative a, so steps between roots (including them) are solutions.

            let min_n = min_n.ceil() as isize;
            let max_n = max_n.floor() as isize;

            let after_xs = (min_n, max_n);
            let after_xe = solve_bounds(x, area.x.1);

            let between_xs_xe;
            if let Some((min_n, _)) = after_xe {
                // You do the same roots search for crossing x_e.
                // This parabola crosses x_e also later (after achieving it's apex), but it's always after domain end (n goes from 1 to x).
                // In fact we are searching for solving:
                // x(n) <= x_e
                // inequality so roots are NOT part of the solution. That's why
                // you are taking ceiling but add small delta (0.0001) to make integer solution go + 1.
                let min_n = (min_n + 0.0001).ceil() as isize;

                between_xs_xe = (after_xs.0, min_n);
            } else {
                // If you are unable to find roots for x_e that means this parabola never reaches x_e so it's open ended after crossing x_s.
                between_xs_xe = (after_xs.0, isize::MAX);
            }

            x_solutions.push((x, between_xs_xe));
        } else {
            continue;
        }
    }

    let mut y = area.y.0;
    while y <= max_y {
        if let Some((_, n_1)) = solve_bounds(y, area.y.1) {
            let n_1i = n_1.ceil() as isize;

            if let Some((_, n_2)) = solve_bounds(y, area.y.0) {
                let n_2i = n_2.floor() as isize;

                // There may be situations where solutions are found which are not containing any integer steps.
                // This check filters such situations out.
                if n_1.floor() == n_2.floor() && n_1.floor() != n_1 && n_2.floor() != n_2 {
                    y += 1;
                    continue;
                }

                let y_start = isize::min(n_1i, n_2i);
                let y_end = isize::max(n_1i, n_2i);

                let mut n = y_start;
                while n <= y_end {
                    let r = n * y - ((n - 1) * n) / 2;

                    if r < area.y.0 || r > area.y.1 {
                        println!("y = {} n = {} not within area {:?} ({})", y, n, area.y, r);
                        println!("{:?} {:?}", (n_1, n_2), (n_1i, n_2i));
                    }

                    n += 1;
                }

                for (_, x_bound) in x_solutions.iter().copied() {
                    if isize::max(y_start, x_bound.0) <= isize::min(y_end, x_bound.1 - 1) {
                        distinct += 1;
                    }
                }
            }
        }

        y += 1;
    }

    println!(
        "Found {} distinct initial velocity values hitting the area",
        distinct
    );

    Ok(())
}

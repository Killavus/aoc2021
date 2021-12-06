use intervaltree::IntervalTree;
use std::{collections::HashSet, ops::RangeInclusive, path::Path};

use anyhow::{anyhow, Result};
use std::fs;

#[derive(Debug, Clone, Copy)]
struct HydrothermalVent {
    start: (usize, usize),
    end: (usize, usize),
}

#[derive(Eq, PartialEq, Debug)]
enum VentOrientation {
    Horizontal,
    Vertical,
    Diagonal,
}

impl HydrothermalVent {
    fn y_axis_overlap(&self, other: &HydrothermalVent) -> bool {
        let (s, e) = (
            usize::min(self.start.1, self.end.1),
            usize::max(self.start.1, self.end.1),
        );

        let (other_s, other_e) = (
            usize::min(other.start.1, other.end.1),
            usize::max(other.start.1, other.end.1),
        );

        (s..=e).contains(&other_s) || (s..=e).contains(&other_e)
    }

    fn orientation(&self) -> VentOrientation {
        if self.start.0 == self.end.0 {
            VentOrientation::Vertical
        } else if self.start.1 == self.end.1 {
            VentOrientation::Horizontal
        } else {
            VentOrientation::Diagonal
        }
    }

    fn ordered_positions(&self) -> ((usize, usize), (usize, usize)) {
        (
            (
                usize::min(self.start.0, self.end.0),
                usize::min(self.start.1, self.end.1),
            ),
            (
                usize::max(self.start.0, self.end.0),
                usize::max(self.start.1, self.end.1),
            ),
        )
    }

    fn points(&self) -> Vec<(usize, usize)> {
        use VentOrientation::*;
        let (start, end) = self.ordered_positions();

        match self.orientation() {
            Horizontal => (start.0..=end.0).map(|x| (x, start.1)).collect(),
            Vertical => (start.1..=end.1).map(|y: usize| (start.0, y)).collect(),
            Diagonal => {
                let x_range = if self.start.0 > self.end.0 {
                    (self.end.0..=self.start.0).rev().collect::<Vec<_>>()
                } else {
                    (self.start.0..=self.end.0).collect::<Vec<_>>()
                };

                let y_range = if self.start.1 > self.end.1 {
                    (self.end.1..=self.start.1).rev().collect::<Vec<_>>()
                } else {
                    (self.start.1..=self.end.1).collect::<Vec<_>>()
                };

                x_range.into_iter().zip(y_range.into_iter()).collect()
            }
        }
    }
}

impl TryFrom<&str> for HydrothermalVent {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut arrow_split = value.split(" -> ");

        let start_pos = arrow_split.next().ok_or(anyhow!("Malformed input"))?;
        let end_pos = arrow_split.next().ok_or(anyhow!("Malformed input"))?;

        let start_pos = start_pos
            .split(",")
            .map(str::parse)
            .collect::<Result<Vec<usize>, _>>()?;

        let end_pos = end_pos
            .split(",")
            .map(str::parse)
            .collect::<Result<Vec<usize>, _>>()?;

        if start_pos.len() < 2 || end_pos.len() < 2 {
            return Err(anyhow!("Malformed input"));
        }

        Ok(Self {
            start: (start_pos[0], start_pos[1]),
            end: (end_pos[0], end_pos[1]),
        })
    }
}

fn read_input(path: impl AsRef<Path>) -> Result<Vec<HydrothermalVent>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .map(HydrothermalVent::try_from)
        .collect::<Result<Vec<_>, _>>()?)
}

/// This is a more sophisticated algorithm to solve this problem.
/// It uses sweeping line approach (sorting by one axis, in this case it is x-axis) and maintains
/// a "working set" of segments to be considered.
/// Then it compares newly processed segment to all items in the working set checking for overlaps on another axis (y-axis).
///
/// Right now this approach is way slower. It is mostly because y-axis overlap part is not optimized at all.
/// A proper data structure (interval tree) may be needed to make this approach optimal.
fn overlapping_vents_sweeping(vents: &[HydrothermalVent]) -> usize {
    // Constructing sweeping structure: O(n * log(n))
    let mut sweep_x = vents
        .iter()
        .enumerate()
        .flat_map(|(i, v)| {
            vec![
                (usize::min(v.start.0, v.end.0), i, false),
                (usize::max(v.start.0, v.end.0), i, true),
            ]
        })
        .collect::<Vec<_>>();

    sweep_x.sort_by_key(|line| (line.0, line.2));

    // Constructing interval tree: O(n * log(n))
    let y_tree: IntervalTree<usize, usize> = vents
        .iter()
        .enumerate()
        .map(|(index, vent)| {
            let (start, end) = vent.ordered_positions();
            ((start.1..end.1 + 1), index)
        })
        .collect();

    let mut working_set: HashSet<usize> = HashSet::new();
    let mut result_set: HashSet<(usize, usize)> = HashSet::new();

    // Maintaining working set: O(n * log(n))
    for (_, segment, segment_end) in sweep_x {
        if segment_end {
            working_set.remove(&segment);
        } else {
            let segment_vent = &vents[segment];
            let (start, end) = segment_vent.ordered_positions();
            // Querying: (O(log(n) + m)), checking for working set: O(mlog(n))
            for overlapping_segment in y_tree.query(start.1..end.1 + 1) {
                if working_set.contains(&overlapping_segment.value) {
                    result_set.extend(
                        vents[overlapping_segment.value]
                            .points()
                            .into_iter()
                            .collect::<HashSet<_>>()
                            .intersection(&segment_vent.points().into_iter().collect()),
                    );
                }
            }

            working_set.insert(segment);
        }
    }

    result_set.len()
}

/// This is basically a brute-force approach to solving this problem. This is not optimised at all, and more sophisticated algorithm is certainly possible.
/// If I'd have to guess, an approach with sweeping algorithm over x-axis and interval tree on y-axis can solve this problem in O(N * log(N)).
fn overlapping_vents_brute(vents: &[HydrothermalVent]) -> usize {
    let max_x = vents.iter().flat_map(|v| vec![v.start.0, v.end.0]).max();
    let max_y = vents.iter().flat_map(|v| vec![v.start.1, v.end.1]).max();

    if let (Some(max_x), Some(max_y)) = (max_x, max_y) {
        let mut board = vec![vec![0; max_x + 1]; max_y + 1];

        for vent in vents {
            vent.points()
                .into_iter()
                .for_each(|(x, y)| board[y][x] += 1)
        }

        board
            .into_iter()
            .map(|row| row.into_iter().filter(|x| *x > 1).count())
            .sum()
    } else {
        0
    }
}

fn main() -> Result<()> {
    let vents = read_input("./input")?;

    println!(
        "Dangerous areas count (without diagonals): {}",
        overlapping_vents_brute(
            &vents
                .iter()
                .copied()
                .filter(|seg| seg.start.0 == seg.end.0 || seg.start.1 == seg.end.1)
                .collect::<Vec<_>>()
        )
    );

    println!(
        "Dangerous areas count (with diagonals): {}",
        overlapping_vents_brute(&vents)
    );

    Ok(())
}

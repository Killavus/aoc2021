use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fmt::Display;
use std::fs;
use std::str::FromStr;

#[derive(Debug)]
enum PageFold {
    FoldY(usize),
    FoldX(usize),
}

#[derive(Debug)]
struct ManualPage {
    dots: Vec<(usize, usize)>,
    folds: Vec<PageFold>,
}

impl FromStr for PageFold {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let without_prefix = s
            .strip_prefix("fold along ")
            .ok_or(anyhow!("Failed to strip prefix - {}", s))?;

        let mut splitted_fold_data = without_prefix.split('=');
        let axis = splitted_fold_data
            .next()
            .ok_or(anyhow!("failed to parse axis data - {}", s))?;
        let point = splitted_fold_data
            .next()
            .ok_or(anyhow!("failed to parse point data - {}", s))?
            .parse()?;

        match axis {
            "y" => Ok(Self::FoldY(point)),
            "x" => Ok(Self::FoldX(point)),
            _ => Err(anyhow!("unknown axis type - {}", s)),
        }
    }
}

struct DotMap(HashSet<(usize, usize)>);

impl Display for DotMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.len() == 0 {
            return write!(f, "<empty result>");
        }

        let max_x = self
            .0
            .iter()
            .max_by_key(|point| point.0)
            .map(|point| point.0);

        let max_y = self
            .0
            .iter()
            .max_by_key(|point| point.1)
            .map(|point| point.1);

        // SAFETY: There is a short-circuit check at the beginning of this function for an empty point cloud.
        let (mx, my) = max_x.zip(max_y).unwrap();

        let mut board = vec![vec!['.'; mx + 1]; my + 1];

        self.0.iter().copied().for_each(|(x, y)| {
            board[y][x] = '#';
        });

        Ok(for row in board.into_iter() {
            for point in row.into_iter() {
                write!(f, "{}", point)?;
            }
            write!(f, "\n")?
        })
    }
}

impl FromStr for ManualPage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut folds: Vec<PageFold> = vec![];
        let mut dots: Vec<(usize, usize)> = vec![];

        for line in s.lines() {
            if line.is_empty() {
                continue;
            }

            if line.starts_with("fold along") {
                folds.push(str::parse(line)?);
            } else {
                let mut point = line.split(',');
                let x = point
                    .next()
                    .ok_or(anyhow!("failed to get point x - {}", line))?
                    .parse()?;
                let y = point
                    .next()
                    .ok_or(anyhow!("failed to get point y - {}", line))?
                    .parse()?;

                dots.push((x, y));
            }
        }

        Ok(Self { folds, dots })
    }
}

impl PageFold {
    fn translate_dot(&self, (x, y): (usize, usize)) -> (usize, usize) {
        use PageFold::*;

        match self {
            FoldX(axis_start) => {
                let mut x = x;
                if x > *axis_start {
                    x = *axis_start - (x - *axis_start);
                }

                (x, y)
            }
            FoldY(axis_start) => {
                let mut y = y;
                if y > *axis_start {
                    y = *axis_start - (y - *axis_start);
                }

                (x, y)
            }
        }
    }
}

impl ManualPage {
    fn final_dots(&self, folds_count: Option<usize>) -> Vec<(usize, usize)> {
        self.dots
            .iter()
            .copied()
            .map(|dot| {
                let folds = self.folds.iter().take(folds_count.unwrap_or(usize::MAX));

                folds.fold(dot, |dot, fold| fold.translate_dot(dot))
            })
            .collect()
    }

    fn count_dots(&self, folds_count: Option<usize>) -> usize {
        HashSet::<(usize, usize)>::from_iter(self.final_dots(folds_count).into_iter())
            .into_iter()
            .count()
    }
}

fn main() -> Result<()> {
    let manual_page: ManualPage = fs::read_to_string("./input")?.parse()?;

    println!(
        "Number of dots after folding one time: {}",
        manual_page.count_dots(Some(1))
    );

    let folded_dots_map = DotMap(HashSet::<(usize, usize)>::from_iter(
        manual_page.final_dots(None).into_iter(),
    ));

    println!("Resulting page after all folding:");
    println!("{}", folded_dots_map);

    Ok(())
}

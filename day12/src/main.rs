use anyhow::{anyhow, Result};
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    fs,
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Cave {
    Small(String),
    Big(String),
    Start,
    End,
}

#[derive(Debug)]
struct CaveSystem(HashMap<Cave, Vec<Cave>>);

impl FromStr for Cave {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "start" => Cave::Start,
            "end" => Cave::End,
            str if str.to_ascii_lowercase() == str => Cave::Small(str.into()),
            str => Cave::Big(str.into()),
        })
    }
}

impl Cave {
    fn can_backtrack(&self) -> bool {
        use Cave::*;

        match self {
            Big(_) => true,
            _ => false,
        }
    }

    fn is_small(&self) -> bool {
        use Cave::*;
        match self {
            Small(_) => true,
            _ => false,
        }
    }
}

impl FromStr for CaveSystem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cave_map = HashMap::new();

        for line in s.lines() {
            let mut connected_caves = line.split('-');
            let cave_a: Cave = connected_caves
                .next()
                .ok_or(anyhow!("Failed to get the first cave - {}", s))?
                .parse()?;
            let cave_b: Cave = connected_caves
                .next()
                .ok_or(anyhow!("Failed to get the second cave - {}", s))?
                .parse()?;

            cave_map
                .entry(cave_a.clone())
                .or_insert(vec![])
                .push(cave_b.clone());
            cave_map.entry(cave_b).or_insert(vec![]).push(cave_a);
        }

        Ok(Self(cave_map))
    }
}

impl CaveSystem {
    fn depth_first<'system>(
        &'system self,
        current: &'system Cave,
        used: &mut HashSet<&'system Cave>,
    ) -> usize {
        let mut result = 0;

        if current == &Cave::End {
            return 1;
        }

        if !current.can_backtrack() {
            used.insert(current);
        }

        for cave in self.0[current].iter() {
            if cave != &Cave::Start && !used.contains(cave) {
                result += self.depth_first(cave, used);
            }
        }

        if used.contains(current) {
            used.remove(current);
        }

        result
    }

    fn depth_first_twice<'system>(
        &'system self,
        current: &'system Cave,
        twice_cave: Option<&'system Cave>,
        used: &mut HashSet<&'system Cave>,
    ) -> usize {
        let mut result = 0;

        if current == &Cave::End {
            return 1;
        }

        if !current.can_backtrack() {
            used.insert(current);
        }

        for cave in self.0[current].iter() {
            if cave != &Cave::Start {
                if used.contains(cave) && cave.is_small() && twice_cave.is_none() {
                    result += self.depth_first_twice(cave, Some(cave), used);
                }

                if !used.contains(cave) {
                    result += self.depth_first_twice(cave, twice_cave, used);
                }
            }
        }

        if used.contains(current) {
            if let Some(twice_cave) = twice_cave {
                if twice_cave != current {
                    used.remove(current);
                }
            } else {
                used.remove(current);
            }
        }

        result
    }

    fn paths_count(&self) -> usize {
        let mut used = HashSet::new();

        self.depth_first(&Cave::Start, &mut used)
    }

    fn paths_count_small_twice(&self) -> usize {
        let mut used = HashSet::new();

        self.depth_first_twice(&Cave::Start, None, &mut used)
    }
}

fn main() -> Result<()> {
    let caves: CaveSystem = fs::read_to_string("./input")?.parse()?;

    println!("Number of paths from start to end: {}", caves.paths_count());
    println!(
        "Number of paths from start to end, entering caves twice: {}",
        caves.paths_count_small_twice()
    );
    Ok(())
}

use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use std::{collections::HashMap, str::FromStr};
use utils::consecutive_pairs;

#[derive(Debug)]
struct PairRule {
    pair: (char, char),
    product: char,
}

impl FromStr for PairRule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pair_product_split = s.split(" -> ");
        let pair = pair_product_split
            .next()
            .ok_or(anyhow!("failed to find pair for pair rule: {}", s))?;
        let product = pair_product_split
            .next()
            .ok_or(anyhow!("failed to find product for pair rule: {}", s))?;

        let product_elements = product.chars().collect::<Vec<_>>();
        let pair_elements = pair.chars().collect::<Vec<_>>();

        Ok(Self {
            pair: (
                pair_elements.get(0).copied().ok_or(anyhow!(
                    "failed to find first element of pair in pair rule: {}",
                    s
                ))?,
                pair_elements.get(1).copied().ok_or(anyhow!(
                    "failed to find second element of pair in pair rule: {}",
                    s
                ))?,
            ),
            product: product_elements
                .get(0)
                .copied()
                .ok_or(anyhow!("failed to find product in pair rule: {}", s))?,
        })
    }
}

fn read_input(path: impl AsRef<Path>) -> Result<(Vec<char>, HashMap<(char, char), char>)> {
    let str = path.as_ref().to_str().map(Into::<String>::into);
    let data = fs::read_to_string(path)?;
    let mut lines = data.lines();

    let first_line = lines.next().ok_or(anyhow!(
        "{:?}: data malformed - first line doesn't exist",
        str
    ))?;

    let chain = first_line.chars().collect();
    let mut ruleset = HashMap::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }

        let pair_rule: PairRule = line.parse()?;
        ruleset.insert(pair_rule.pair, pair_rule.product);
    }

    Ok((chain, ruleset))
}

fn simulate_polymerisation(
    starting_polymer: &[char],
    ruleset: &HashMap<(char, char), char>,
    steps: usize,
) -> usize {
    let mut elements_counter: HashMap<char, usize> = HashMap::new();
    starting_polymer.iter().copied().for_each(|element| {
        *elements_counter.entry(element).or_default() += 1;
    });
    let mut producing_pairs: HashMap<(char, char), usize> = HashMap::with_capacity(ruleset.len());

    for pair in consecutive_pairs(starting_polymer.iter().copied()) {
        if ruleset.contains_key(&pair) {
            *producing_pairs.entry(pair).or_default() += 1;
        }
    }

    for _ in 0..steps {
        let mut new_producing_pairs = HashMap::with_capacity(ruleset.len());
        for (pair, count) in producing_pairs.into_iter() {
            let product = ruleset[&pair];
            let (substrate_a, substrate_b) = pair;
            let result_first = (substrate_a, product);
            let result_second = (product, substrate_b);

            *elements_counter.entry(product).or_default() += count;

            if ruleset.contains_key(&result_first) {
                *new_producing_pairs.entry(result_first).or_default() += count;
            }

            if ruleset.contains_key(&result_second) {
                *new_producing_pairs.entry(result_second).or_default() += count;
            }
        }
        producing_pairs = new_producing_pairs;
    }

    let most_occuring_element = elements_counter
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(_, count)| count)
        .copied();
    let least_occuring_element = elements_counter
        .iter()
        .min_by_key(|(_, count)| *count)
        .map(|(_, count)| count)
        .copied();

    if let Some((max, min)) = most_occuring_element.zip(least_occuring_element) {
        max - min
    } else {
        panic!("Invalid analysis - empty chain");
    }
}

fn main() -> Result<()> {
    let (starting_polymer, ruleset) = read_input("./input")?;

    println!(
        "Quantity analysis after 10 polymerisation steps: {}",
        simulate_polymerisation(&starting_polymer, &ruleset, 10)
    );

    println!(
        "Quantity analysis after 40 polymerisation steps: {}",
        simulate_polymerisation(&starting_polymer, &ruleset, 40)
    );

    Ok(())
}

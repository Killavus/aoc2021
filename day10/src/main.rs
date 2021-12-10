use std::error::Error;
use std::fs;

mod parser;
use parser::{NavigationLineParser, ParserResult};

fn total_syntax_error_score(navigation_subsystem: &str) -> usize {
    navigation_subsystem
        .lines()
        .map(Into::<NavigationLineParser>::into)
        .map(NavigationLineParser::parse)
        .flat_map(|result| result.first_illegal())
        .map(|bracket| match bracket {
            ')' => 3,
            ']' => 57,
            '}' => 1197,
            '>' => 25137,
            _ => 0,
        })
        .sum()
}

fn total_autocompletion_score(navigation_subsystem: &str) -> usize {
    let mut completions: Vec<usize> = navigation_subsystem
        .lines()
        .map(Into::<NavigationLineParser>::into)
        .map(NavigationLineParser::parse)
        .flat_map(ParserResult::completion)
        .map(|completion| {
            completion.into_iter().fold(0, |score, bracket| {
                let bracket_score = match bracket {
                    ')' => 1,
                    ']' => 2,
                    '}' => 3,
                    '>' => 4,
                    _ => 0,
                };

                score * 5 + bracket_score
            })
        })
        .collect();

    completions.sort_unstable();
    completions[completions.len() / 2]
}

fn main() -> Result<(), Box<dyn Error>> {
    let navigation_subsystem = fs::read_to_string("./input")?;

    println!(
        "Total syntax error score for navigation subsystem: {}",
        total_syntax_error_score(&navigation_subsystem)
    );

    println!(
        "Total autocompletion score for navigation subsystem: {}",
        total_autocompletion_score(&navigation_subsystem)
    );

    Ok(())
}

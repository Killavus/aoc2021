use std::error::Error;
use std::fs;

struct NavigationLineParser<'a> {
    line: &'a str,
    bracket_stack: Vec<char>,
}

#[derive(Debug, PartialEq, Eq)]
enum ParserResult {
    Corrupted(char),
    Incomplete(Vec<char>),
}

impl ParserResult {
    fn first_illegal(&self) -> Option<char> {
        use ParserResult::*;
        match self {
            Corrupted(ch) => Some(*ch),
            Incomplete(_) => None,
        }
    }

    fn completion(self) -> Option<Vec<char>> {
        use ParserResult::*;

        match self {
            Corrupted(_) => None,
            Incomplete(bracket_stack) => Some(
                bracket_stack
                    .into_iter()
                    .rev()
                    .map(|bracket| match bracket {
                        '(' => ')',
                        '[' => ']',
                        '{' => '}',
                        '<' => '>',
                        _ => 'E',
                    })
                    .collect(),
            ),
        }
    }
}

impl<'line> From<&'line str> for NavigationLineParser<'line> {
    fn from(line: &'line str) -> Self {
        NavigationLineParser::new(line)
    }
}

impl<'line> NavigationLineParser<'line> {
    const OPENING_BRACKETS: [char; 4] = ['(', '{', '<', '['];
    const CLOSING_BRACKETS: [char; 4] = [')', '}', '>', ']'];

    fn new(line: &'line str) -> Self {
        Self {
            line,
            bracket_stack: Vec::with_capacity(line.len()),
        }
    }

    fn parse(mut self) -> ParserResult {
        for bracket in self.line.chars() {
            if Self::OPENING_BRACKETS.contains(&bracket) {
                self.bracket_stack.push(bracket);
            } else if Self::CLOSING_BRACKETS.contains(&bracket) {
                let opening_bracket = self.bracket_stack.last();

                match opening_bracket {
                    Some(opening_bracket) => {
                        let index = Self::OPENING_BRACKETS
                            .iter()
                            .position(|bracket| bracket == opening_bracket)
                            .unwrap();

                        // We found closing bracket which is invalid - this line is corrupted.
                        if Self::CLOSING_BRACKETS[index] != bracket {
                            return ParserResult::Corrupted(bracket);
                        }
                    }
                    None => {
                        // We found closing bracket and there is no matching opening bracket at all
                        // - this line is corrupted.
                        return ParserResult::Corrupted(bracket);
                    }
                }

                self.bracket_stack.pop();
            } else {
                panic!("Invalid input - {} exists in navigation line", bracket);
            }
        }

        ParserResult::Incomplete(self.bracket_stack)
    }
}

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

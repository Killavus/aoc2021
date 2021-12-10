pub struct NavigationLineParser<'a> {
    line: &'a str,
    bracket_stack: Vec<char>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParserResult {
    Corrupted(char),
    Incomplete(Vec<char>),
}

impl ParserResult {
    pub fn first_illegal(&self) -> Option<char> {
        use ParserResult::*;
        match self {
            Corrupted(ch) => Some(*ch),
            Incomplete(_) => None,
        }
    }

    pub fn completion(self) -> Option<Vec<char>> {
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

    pub fn parse(mut self) -> ParserResult {
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

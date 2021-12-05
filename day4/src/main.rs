use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::str::Lines;

#[derive(Debug)]
struct BingoBoard {
    board: HashMap<usize, (usize, usize)>,
    col_sums: [(usize, usize); 5],
    row_sums: [(usize, usize); 5],
    won: bool,
}

struct BingoBoardBuilder {
    current_row: usize,
    board: HashMap<usize, (usize, usize)>,
    col_sums: [(usize, usize); 5],
    row_sums: [(usize, usize); 5],
}

impl BingoBoardBuilder {
    pub fn new() -> Self {
        Self {
            current_row: 0,
            board: HashMap::with_capacity(25),
            col_sums: [(0, 0); 5],
            row_sums: [(0, 0); 5],
        }
    }

    pub fn build(self) -> Option<BingoBoard> {
        if self.is_complete() {
            Some(BingoBoard {
                board: self.board,
                col_sums: self.col_sums,
                row_sums: self.row_sums,
                won: false,
            })
        } else {
            None
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_row == 5
    }

    pub fn fill_row(&mut self, row: impl Iterator<Item = usize>) {
        for (col, number) in row.enumerate() {
            self.board.insert(number, (self.current_row, col));
            self.row_sums[self.current_row].0 += number;
            self.col_sums[col].0 += number;
        }

        self.current_row += 1;
    }
}

impl<'a> TryFrom<&mut Lines<'a>> for BingoBoard {
    type Error = io::Error;

    fn try_from(lines: &mut Lines<'a>) -> Result<Self, Self::Error> {
        let mut builder = BingoBoardBuilder::new();

        while !builder.is_complete() {
            match lines.next() {
                Some(line) => {
                    if line.is_empty() {
                        continue;
                    } else {
                        builder.fill_row(line.split_ascii_whitespace().flat_map(str::parse));
                    }
                }
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "failed to complete bingo board from input",
                    ))
                }
            }
        }

        Ok(builder.build().expect("error in builder implementation"))
    }
}

impl BingoBoard {
    pub fn mark(&mut self, number: usize) -> Option<usize> {
        if !self.won {
            if let Some((row, col)) = self.board.get(&number).copied() {
                self.col_sums[col].1 += number;
                self.row_sums[row].1 += number;

                if self.col_sums[col].0 == self.col_sums[col].1
                    || self.row_sums[row].0 == self.row_sums[row].1
                {
                    let score = self.col_sums.iter().map(|pair| pair.0).sum::<usize>()
                        - self.col_sums.iter().map(|pair| pair.1).sum::<usize>();
                    self.won = true;
                    Some(score * number)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn read_guesses<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Option<Vec<usize>> {
    lines
        .next()
        .map(|line| line.split(',').flat_map(str::parse).collect())
}

fn read_input(path: impl AsRef<Path>) -> Result<(Vec<usize>, Vec<BingoBoard>), io::Error> {
    let data = fs::read_to_string(path)?;
    let mut data_lines = data.lines();

    let guesses = read_guesses(&mut data_lines).ok_or(io::Error::new(
        io::ErrorKind::InvalidInput,
        "failed to find guesses line",
    ))?;

    let mut boards = vec![];
    while let Ok(board) = BingoBoard::try_from(&mut data_lines) {
        boards.push(board);
    }

    Ok((guesses, boards))
}

fn main() -> Result<(), Box<dyn Error>> {
    let (guesses, mut boards) = read_input("./input")?;

    let mut boards_won = 0;
    let total_boards = boards.len();

    for guess in guesses.iter().copied() {
        boards.iter_mut().for_each(|board| {
            let mark_score = board.mark(guess);
            mark_score.iter().for_each(|score| {
                boards_won += 1;
                if boards_won == 1 {
                    println!("First board's final score is: {}", score);
                } else if boards_won == total_boards {
                    println!("Last board's final score is: {}", score);
                }
            });
        });

        if boards_won == total_boards {
            break;
        }
    }

    Ok(())
}

use std::{collections::HashMap, io, str::Lines};

#[derive(Debug)]
pub struct BingoBoard {
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
    fn new() -> Self {
        Self {
            current_row: 0,
            board: HashMap::with_capacity(25),
            col_sums: [(0, 0); 5],
            row_sums: [(0, 0); 5],
        }
    }

    fn build(self) -> Option<BingoBoard> {
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

    fn is_complete(&self) -> bool {
        self.current_row == 5
    }

    fn fill_row(&mut self, row: impl Iterator<Item = usize>) {
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
    /// Marks a given number and checks for board's win condition.
    ///
    /// In case of marking an already completed board the move is ignored and `None` is returned immediately.
    ///
    /// # Returns:
    /// `None` if this move does not complete the board, or Some(score) otherwise.
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

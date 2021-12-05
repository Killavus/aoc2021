use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

mod bingo;

use bingo::BingoBoard;

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

fn simulate_bingo_game(guesses: &[usize], boards: &mut [BingoBoard]) -> (usize, usize) {
    let mut boards_won = 0;
    let total_boards = boards.len();
    let mut result = (0, 0);

    for guess in guesses.iter().copied() {
        boards.iter_mut().for_each(|board| {
            let mark_score = board.mark(guess);
            mark_score.iter().copied().for_each(|score| {
                boards_won += 1;
                if boards_won == 1 {
                    result.0 = score;
                } else if boards_won == total_boards {
                    result.1 = score;
                }
            });
        });

        if boards_won == total_boards {
            break;
        }
    }

    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let (guesses, mut boards) = read_input("./input")?;
    let (first_won_score, last_won_score) = simulate_bingo_game(&guesses, &mut boards);

    println!(
        "As a player, your winning board's score is {}.",
        first_won_score
    );
    println!(
        "As a squid, your winning board's score is {}.",
        last_won_score
    );

    Ok(())
}

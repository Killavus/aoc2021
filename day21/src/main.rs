use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::{convert::Infallible, str::FromStr};

struct GameState {
    one_pos: u64,
    two_pos: u64,
}

impl FromStr for GameState {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let one_pos: u64 = lines
            .next()
            .unwrap()
            .strip_prefix("Player 1 starting position: ")
            .unwrap()
            .parse()
            .unwrap();
        let two_pos: u64 = lines
            .next()
            .unwrap()
            .strip_prefix("Player 2 starting position: ")
            .unwrap()
            .parse()
            .unwrap();

        Ok(Self { one_pos, two_pos })
    }
}

struct GameScore {
    p1_score: u64,
    p2_score: u64,
    rolls: u64,
}

struct DeterministicDice {
    value: u64,
    rolls: u64,
}

impl DeterministicDice {
    fn new() -> Self {
        Self { value: 1, rolls: 0 }
    }

    fn roll(&mut self) -> u64 {
        let val = self.value;

        self.value += 1;
        self.rolls += 1;

        if self.value == 101 {
            self.value = 1;
        }

        val
    }
}

fn simulate_game(initial: &GameState) -> GameScore {
    let mut p1_score = 0;
    let mut p2_score = 0;
    let mut p1_pos = initial.one_pos - 1;
    let mut p2_pos = initial.two_pos - 1;
    let mut p1_turn = true;

    let mut dice = DeterministicDice::new();

    while p1_score < 1000 && p2_score < 1000 {
        if p1_turn {
            p1_pos += dice.roll() + dice.roll() + dice.roll();
            p1_pos %= 10;
            p1_score += p1_pos + 1;
        } else {
            p2_pos += dice.roll() + dice.roll() + dice.roll();
            p2_pos %= 10;
            p2_score += p2_pos + 1;
        }

        p1_turn = !p1_turn;
    }

    GameScore {
        p1_score,
        p2_score,
        rolls: dice.rolls,
    }
}

const POSSIBLE_ROLLS: [(u64, u64); 7] = [(3, 1), (4, 3), (5, 6), (6, 7), (7, 6), (8, 3), (9, 1)];

// We just memoize game states aggressively.
fn count_states(
    p1_pos: u64,
    p2_pos: u64,
    p1_score: u64,
    p2_score: u64,
    memo: &mut HashMap<(u64, u64, u64, u64), (u64, u64)>,
) -> (u64, u64) {
    if let Some(score) = memo.get(&(p1_pos, p2_pos, p1_score, p2_score)) {
        *score
    } else if p1_score >= 21 {
        (1, 0)
    } else if p2_score >= 21 {
        (0, 1)
    } else {
        let state = (p1_pos, p2_pos, p1_score, p2_score);

        for (add, freq) in POSSIBLE_ROLLS {
            let p1_npos = (p1_pos + add) % 10;
            let p2_npos = p2_pos;
            let p1_nscore = p1_score + p1_npos + 1;
            let p2_nscore = p2_score;

            let subtree = count_states(p2_npos, p1_npos, p2_nscore, p1_nscore, memo);
            let entry = memo.entry(state).or_insert((0, 0));
            entry.0 += freq * subtree.1;
            entry.1 += freq * subtree.0;
        }

        *memo.get(&state).unwrap()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let state: GameState = fs::read_to_string("./input")?.parse()?;

    let GameScore {
        p1_score,
        p2_score,
        rolls,
    } = simulate_game(&state);

    println!(
        "Losing player score * number of dice rolls: {}",
        if p1_score > p2_score {
            p2_score * rolls
        } else {
            p1_score * rolls
        }
    );

    println!(
        "Counting states: {:?}",
        count_states(
            state.one_pos - 1,
            state.two_pos - 1,
            0,
            0,
            &mut HashMap::default()
        )
    );

    Ok(())
}

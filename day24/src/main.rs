use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use std::fs;
use std::str::{FromStr, SplitAsciiWhitespace};

enum Operand {
    W,
    X,
    Y,
    Z,
    C(i64),
}

#[derive(Clone, Debug)]
struct Memory {
    w: i64,
    x: i64,
    y: i64,
    z: i64,
}

impl Memory {
    fn reg(&mut self, operand: &Operand) -> &mut i64 {
        use Operand::*;

        match operand {
            X => &mut self.x,
            Y => &mut self.y,
            Z => &mut self.z,
            W => &mut self.w,
            _ => panic!("Attempt to take const as register"),
        }
    }

    fn value(&self, operand: &Operand) -> i64 {
        use Operand::*;

        match operand {
            X => self.x,
            Y => self.y,
            Z => self.z,
            W => self.w,
            C(i) => *i,
        }
    }
}

enum Operation {
    Inp(Operand),
    Add(Operand, Operand),
    Mul(Operand, Operand),
    Div(Operand, Operand),
    Mod(Operand, Operand),
    Eql(Operand, Operand),
}

struct Program(Vec<Operation>);

impl FromStr for Operation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn operand(s: &str, parts: &mut SplitAsciiWhitespace<'_>, i: usize) -> Result<Operand> {
            let operand = parts
                .next()
                .ok_or_else(|| anyhow!("Failed to get operand {}: {}", i, s))?;

            Ok(match operand {
                "x" => Operand::X,
                "y" => Operand::Y,
                "w" => Operand::W,
                "z" => Operand::Z,
                c => Operand::C(c.parse().map_err(Into::<anyhow::Error>::into)?),
            })
        }

        fn two_operands(
            s: &str,
            parts: &mut SplitAsciiWhitespace<'_>,
        ) -> Result<(Operand, Operand)> {
            Ok((operand(s, parts, 1)?, operand(s, parts, 2)?))
        }

        let mut parts = s.split_ascii_whitespace();

        let operation = parts
            .next()
            .ok_or_else(|| anyhow!("Failed to read operation type: {}", s))?;

        match operation {
            "inp" => {
                let op = operand(s, &mut parts, 1)?;
                Ok(Operation::Inp(op))
            }
            "add" => {
                let (op1, op2) = two_operands(s, &mut parts)?;
                Ok(Operation::Add(op1, op2))
            }
            "mul" => {
                let (op1, op2) = two_operands(s, &mut parts)?;
                Ok(Operation::Mul(op1, op2))
            }
            "div" => {
                let (op1, op2) = two_operands(s, &mut parts)?;
                Ok(Operation::Div(op1, op2))
            }
            "mod" => {
                let (op1, op2) = two_operands(s, &mut parts)?;
                Ok(Operation::Mod(op1, op2))
            }
            "eql" => {
                let (op1, op2) = two_operands(s, &mut parts)?;
                Ok(Operation::Eql(op1, op2))
            }
            _ => {
                return Err(anyhow!("Unknown operation type: {}", s));
            }
        }
    }
}

impl FromStr for Program {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.lines().map(str::parse).collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

fn digits_of(n: i64) -> Vec<i64> {
    let mut r = n;
    let mut digits = vec![];
    while r != 0 {
        digits.push((r % 10) as i64);
        r /= 10;
    }

    digits
}

struct ArithmeticLogicUnit;
impl ArithmeticLogicUnit {
    const EXC_MEM: Memory = Memory {
        x: 0,
        y: 0,
        z: 1,
        w: 0,
    };

    fn execute(&self, program: &Program, mut tape: VecDeque<i64>) -> Memory {
        let mut mem = Memory {
            x: 0,
            y: 0,
            z: 0,
            w: 0,
        };

        for op in program.0.iter() {
            use Operation::*;

            match op {
                Add(a, b) => {
                    let b_val = mem.value(b);
                    *mem.reg(a) += b_val;
                }
                Mul(a, b) => {
                    let b_val = mem.value(b);
                    *mem.reg(a) *= b_val;
                }
                Mod(a, b) => {
                    let b_val = mem.value(b);
                    if b_val == 0 {
                        return Self::EXC_MEM;
                    }

                    *mem.reg(a) %= b_val;
                }
                Div(a, b) => {
                    let b_val = mem.value(b);
                    if b_val == 0 {
                        return Self::EXC_MEM;
                    }

                    *mem.reg(a) /= b_val;
                }
                Eql(a, b) => {
                    let a_val = mem.value(a);
                    let b_val = mem.value(b);

                    if a_val == b_val {
                        *mem.reg(a) = 1;
                    } else {
                        *mem.reg(a) = 0;
                    }
                }
                Inp(a) => {
                    if tape.is_empty() {
                        return Self::EXC_MEM;
                    }

                    *mem.reg(a) = tape.pop_front().unwrap();
                }
            }
        }

        mem
    }
}

const SUBROUTINE_COEFFS: [(i64, i64); 14] = [
    (12, 6),
    (10, 6),
    (13, 3),
    (-11, 11),
    (13, 9),
    (-1, 3),
    (10, 13),
    (11, 6),
    (0, 14),
    (10, 10),
    (-5, 12),
    (-16, 10),
    (-7, 11),
    (-11, 15),
];

const REMAINING_DROPS: [i64; 14] = [7, 7, 7, 7, 6, 6, 5, 4, 4, 4, 4, 3, 2, 1];

/// This solution exploits the fact that the input data is very structured & unique.
// Basically for every digit there are following two programs used to calculate result:
// prog a: if (z % 26 + px == w) { z } else { 26 * z + py + w }
// prog b: if (z % 26 + px == w) { z / 26 } else { 26 * (z / 26) + py + w }
// prog a is used for digits 1, 2, 3, 5, 7, 8, 10 in my input.
// prog b is used for rest of digits.
// Only z register 'live' between reads of digits to w. So basically we can do depth-first
// search of solution space, trying digits and applying prog a/b logic accordingly.
// key optimisation here to avoid 10**14 search is seeing that only program b can reduce z result
// and only else-branch of prog a can significantly increase z result. We keep track of remaining
// possibilities of 'reducing' the result through program b and how many times we've significantly
// increased the result and bail out early if we cannot reduce the result (number of remaining reductions
// is higher than number of increases we did).
fn search_solution_space(
    depth: usize,
    bad_branches: usize,
    search_order: &[i64],
    z: i64,
    w: i64,
    path: &mut [i64; 14],
) -> bool {
    if depth == 15 {
        return z == 0;
    } else if bad_branches > REMAINING_DROPS[depth - 1] as usize {
        return false;
    } else {
        path[depth - 1] = w;
        for new_w in search_order {
            let c = z % 26 + SUBROUTINE_COEFFS[depth - 1].0;
            if c == w {
                let result = match depth {
                    1 | 2 | 3 | 5 | 7 | 8 | 10 => search_solution_space(
                        depth + 1,
                        bad_branches,
                        search_order,
                        z,
                        *new_w,
                        path,
                    ),
                    _ => search_solution_space(
                        depth + 1,
                        bad_branches - 1,
                        search_order,
                        z / 26,
                        *new_w,
                        path,
                    ),
                };

                if result {
                    return true;
                }
            } else {
                let result = match depth {
                    1 | 2 | 3 | 5 | 7 | 8 | 10 => search_solution_space(
                        depth + 1,
                        bad_branches + 1,
                        search_order,
                        26 * z + w + SUBROUTINE_COEFFS[depth - 1].1,
                        *new_w,
                        path,
                    ),
                    _ => search_solution_space(
                        depth + 1,
                        bad_branches,
                        search_order,
                        26 * (z / 26) + w + SUBROUTINE_COEFFS[depth - 1].1,
                        *new_w,
                        path,
                    ),
                };

                if result {
                    return true;
                }
            }
        }
    }

    false
}

fn main() -> Result<()> {
    let mut path: [i64; 14] = [0; 14];
    let prog: Program = fs::read_to_string("./input")?.parse()?;
    let alu = ArithmeticLogicUnit;

    let highest_first_search_order = [9, 8, 7, 6, 5, 4, 3, 2, 1];
    for w in highest_first_search_order.iter().copied() {
        let result = search_solution_space(1, 0, &highest_first_search_order, 0, w, &mut path);

        if result {
            println!("Highest valid model number is: {:?}", path);
            break;
        }
    }

    println!(
        "Executing MONAD for highest model number: {:?}",
        alu.execute(&prog, {
            let mut r = VecDeque::new();
            r.extend(&path);
            r
        })
    );

    let lowest_first_search_order: [i64; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    for w in lowest_first_search_order.iter().copied() {
        let result = search_solution_space(1, 0, &lowest_first_search_order, 0, w, &mut path);

        if result {
            println!("Lowest valid model number is: {:?}", path);
            break;
        }
    }

    println!(
        "Executing MONAD for lowest model number: {:?}",
        alu.execute(&prog, {
            let mut r = VecDeque::new();
            r.extend(&path);
            r
        })
    );

    Ok(())
}

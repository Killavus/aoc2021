use anyhow::anyhow;
use std::str::FromStr;

pub struct NoteEntry {
    signal_patterns: Vec<String>,
    output_value: Vec<String>,
}

fn ascii_ord(ch: char) -> usize {
    (ch as u8 - 'a' as u8) as usize
}

impl NoteEntry {
    pub fn unique_segments_digits_count(&self) -> usize {
        let unique_segments_count = [2, 3, 4, 7];
        self.output_value
            .iter()
            .map(String::len)
            .filter(|segment_count| unique_segments_count.contains(&segment_count))
            .count()
    }

    fn unscramble(&self) -> [char; 7] {
        let mut result = ['x'; 7];

        let one = self
            .signal_patterns
            .iter()
            .find(|x| x.len() == 2)
            .expect("cannot find one in signal patterns");

        let seven = self
            .signal_patterns
            .iter()
            .find(|x| x.len() == 3)
            .expect("cannot find seven in signal patterns");

        // a is the only one which exists in 7 but not in 1.
        let a_substitute = seven
            .chars()
            .find(|segment| !one.contains(*segment))
            .expect("cannot find unique letter between one and seven");

        let four = self
            .signal_patterns
            .iter()
            .find(|segment| segment.len() == 4)
            .expect("cannot find four in signal patterns");

        let six_segmented = self
            .signal_patterns
            .iter()
            .filter(|segment| segment.len() == 6);

        // number 9 is the only one from 6-segmented that fully includes 4.
        let nine = six_segmented
            .clone()
            .find(|potential_nine| {
                four.chars()
                    .all(|four_segment| potential_nine.contains(four_segment))
            })
            .expect("cannot find nine in signal patterns");

        // number 6 is the only one from 6-segmented that does not fully include 1.
        let six = six_segmented
            .clone()
            .find(|potential_six| !one.chars().all(|segment| potential_six.contains(segment)))
            .expect("cannot find six in signal patterns");

        // the last 6-segmented digit is zero.
        let zero = six_segmented
            .clone()
            .find(|potential_zero| potential_zero != &six && potential_zero != &nine)
            .expect("cannot find zero in signal patterns");

        let eight = self
            .signal_patterns
            .iter()
            .find(|x| x.len() == 7)
            .expect("cannot find eight in signal patterns");

        // c is the only letter missing from six when comparing to eight
        let c_substitute = eight
            .chars()
            .find(|potential_c| !six.contains(*potential_c))
            .expect("cannot find unique letter between eight and nine");

        // d is the only letter missing from zero when comparing to eight
        let d_substitute = eight
            .chars()
            .find(|potential_d| !zero.contains(*potential_d))
            .expect("cannot find unique letter between eight and zero");

        // e is the only letter missing from nine when comparing to eight
        let e_substitute = eight
            .chars()
            .find(|potential_e| !nine.contains(*potential_e))
            .expect("cannot find unique letter between eight and nine");

        // knowing c we can recognize f from one
        let f_substitute = one
            .chars()
            .find(|f_candidate| *f_candidate != c_substitute)
            .expect("cannot find different segment than c in one");

        // knowing c, d, f we can recognize b from four
        let b_substitute = four
            .chars()
            .find(|b_candidate| ![c_substitute, d_substitute, f_substitute].contains(b_candidate))
            .expect("cannot find different segment than c, d, f in four");

        result[ascii_ord(a_substitute)] = 'a';
        result[ascii_ord(b_substitute)] = 'b';
        result[ascii_ord(c_substitute)] = 'c';
        result[ascii_ord(d_substitute)] = 'd';
        result[ascii_ord(e_substitute)] = 'e';
        result[ascii_ord(f_substitute)] = 'f';

        // g is the only missing segment:
        let g_substitute_idx = result
            .iter()
            .position(|ch| *ch == 'x')
            .expect("cannot find missing mapping");

        result[g_substitute_idx] = 'g';

        result
    }

    pub fn unscrambled_output_value(&self) -> usize {
        let proper_wiring = self.unscramble();
        let proper_digits = self.output_value.iter().cloned().map(|digit| {
            let mut unscrambled_segments = digit
                .chars()
                .map(|segment| proper_wiring[(segment as u8 - 'a' as u8) as usize])
                .collect::<Vec<_>>();

            unscrambled_segments.sort_unstable();
            String::from_iter(unscrambled_segments)
        });

        proper_digits
            .into_iter()
            .enumerate()
            .map(|(pos, segment)| {
                self.unscrambled_segments_to_digit(&segment) * (10 as usize).pow((3 - pos) as u32)
            })
            .sum()
    }

    fn unscrambled_segments_to_digit(&self, unscrambled_segment: &str) -> usize {
        match unscrambled_segment {
            "abcefg" => 0,
            "cf" => 1,
            "acdeg" => 2,
            "acdfg" => 3,
            "bcdf" => 4,
            "abdfg" => 5,
            "abdefg" => 6,
            "acf" => 7,
            "abcdefg" => 8,
            "abcdfg" => 9,
            _ => {
                panic!(
                    "non-safe string supplied to private function: {}",
                    unscrambled_segment
                );
            }
        }
    }
}

impl FromStr for NoteEntry {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut entry_parts = s.split('|');
        let signal_patterns = entry_parts
            .next()
            .ok_or(anyhow!("Failed to find signal patterns part"))?
            .split_ascii_whitespace()
            .map(ToOwned::to_owned)
            .collect();
        let output_value = entry_parts
            .next()
            .ok_or(anyhow!("Failed to find output value part"))?
            .split_ascii_whitespace()
            .map(ToOwned::to_owned)
            .collect();

        Ok(Self {
            signal_patterns,
            output_value,
        })
    }
}

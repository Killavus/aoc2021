use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::ops::RangeInclusive;
use std::{convert::Infallible, str::FromStr};
struct EnhancementPixel([u16; 512]);
struct InputImage {
    data: HashSet<(i64, i64)>,
}

struct TrenchMap(EnhancementPixel, InputImage, usize);

impl FromStr for EnhancementPixel {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut algorithm = [0; 512];

        for (i, bit) in s.chars().enumerate() {
            algorithm[i] = if bit == '#' { 1 } else { 0 };
        }

        Ok(Self(algorithm))
    }
}

impl FromStr for InputImage {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = HashSet::new();

        for (y, l) in s.lines().enumerate() {
            for (x, c) in l.chars().enumerate() {
                if c == '#' {
                    data.insert((x as i64, y as i64));
                }
            }
        }

        Ok(Self { data })
    }
}

impl FromStr for TrenchMap {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let first = lines.next().unwrap();
        let algorithm: EnhancementPixel = first.parse()?;

        lines.next().unwrap();

        let mut s = String::new();

        lines.for_each(|l| {
            s.push_str(l);
            s.push_str("\n");
        });

        let rest = s.trim_end();
        let input_image: InputImage = rest.parse()?;

        Ok(Self(algorithm, input_image, 0))
    }
}

impl TrenchMap {
    fn pixel(&self, x: i64, y: i64, x_r: &RangeInclusive<i64>, y_r: &RangeInclusive<i64>) -> u16 {
        if !x_r.contains(&x) || !y_r.contains(&y) {
            (self.2 & 1 & self.0 .0[0] as usize) as u16
        } else {
            if self.1.data.contains(&(x, y)) {
                1
            } else {
                0
            }
        }
    }

    fn convolve(
        &self,
        x: i64,
        y: i64,
        x_r: &RangeInclusive<i64>,
        y_r: &RangeInclusive<i64>,
    ) -> u16 {
        let index: usize = [
            self.pixel(x - 1, y - 1, x_r, y_r) * 256,
            self.pixel(x, y - 1, x_r, y_r) * 128,
            self.pixel(x + 1, y - 1, x_r, y_r) * 64,
            self.pixel(x - 1, y, x_r, y_r) * 32,
            self.pixel(x, y, x_r, y_r) * 16,
            self.pixel(x + 1, y, x_r, y_r) * 8,
            self.pixel(x - 1, y + 1, x_r, y_r) * 4,
            self.pixel(x, y + 1, x_r, y_r) * 2,
            self.pixel(x + 1, y + 1, x_r, y_r),
        ]
        .into_iter()
        .sum::<u16>() as usize;

        self.0 .0[index]
    }

    fn enhance(&mut self) -> usize {
        let mut new_data = HashSet::new();
        let min_x = self.1.data.iter().min_by_key(|p| p.0).map(|p| p.0).unwrap();
        let max_x = self.1.data.iter().max_by_key(|p| p.0).map(|p| p.0).unwrap();
        let min_y = self.1.data.iter().min_by_key(|p| p.1).map(|p| p.1).unwrap();
        let max_y = self.1.data.iter().max_by_key(|p| p.1).map(|p| p.1).unwrap();

        let x_r = min_x..=max_x;
        let y_r = min_y..=max_y;

        let x_r_ex = (min_x - 1)..=(max_x + 1);

        for x in x_r_ex {
            let y_r_ex = (min_y - 1)..=(max_y + 1);

            for y in y_r_ex {
                let pixel = self.convolve(x, y, &x_r, &y_r);

                if pixel == 1 {
                    new_data.insert((x, y));
                }
            }
        }

        self.1.data = new_data;
        self.2 += 1;
        self.1.data.len()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut map: TrenchMap = fs::read_to_string("./input")?.parse()?;

    map.enhance();
    let lit_pixels = map.enhance();

    println!(
        "After enhancing the image twice there are {} pixels lit.",
        lit_pixels
    );

    let mut result = 0;
    for _ in 0..48 {
        result = map.enhance();
    }

    println!(
        "After enhancing the image 50 times there are {} pixels lit.",
        result
    );

    Ok(())
}

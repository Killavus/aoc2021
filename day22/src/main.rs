use anyhow::anyhow;
use std::fs;
use std::str::{FromStr, Split};

#[derive(Debug, Copy, Clone)]
enum RebootInstruction {
    On,
    Off,
}

impl RebootInstruction {
    fn inverse(&self) -> RebootInstruction {
        match self {
            &RebootInstruction::On => RebootInstruction::Off,
            &RebootInstruction::Off => RebootInstruction::On,
        }
    }
}

#[derive(Debug, Clone)]
struct RebootCuboid {
    x0: isize,
    x1: isize,
    y0: isize,
    y1: isize,
    z0: isize,
    z1: isize,
    instruction: RebootInstruction,
}

impl RebootCuboid {
    fn intersect(&self, other: &Self) -> Option<Self> {
        let x0 = isize::max(self.x0, other.x0);
        let x1 = isize::min(self.x1, other.x1);
        let y0 = isize::max(self.y0, other.y0);
        let y1 = isize::min(self.y1, other.y1);
        let z0 = isize::max(self.z0, other.z0);
        let z1 = isize::min(self.z1, other.z1);

        if x0 > x1 || y0 > y1 || z0 > z1 {
            None
        } else {
            Some(Self {
                x0,
                x1,
                y0,
                y1,
                z0,
                z1,
                instruction: self.instruction.inverse(),
            })
        }
    }

    fn in_limit(&self, limit: &Option<(isize, isize)>) -> bool {
        if let Some((min, max)) = limit.iter().copied().next() {
            (min..=max).contains(&self.x0)
                && (min..=max).contains(&self.x1)
                && (min..=max).contains(&self.y0)
                && (min..=max).contains(&self.y1)
                && (min..=max).contains(&self.z0)
                && (min..=max).contains(&self.z1)
        } else {
            true
        }
    }

    fn area(&self) -> isize {
        let result = (self.x1 - self.x0 + 1) * (self.y1 - self.y0 + 1) * (self.z1 - self.z0 + 1);

        match self.instruction {
            RebootInstruction::Off => -result,
            RebootInstruction::On => result,
        }
    }
}

impl FromStr for RebootCuboid {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_dim(
            split: &mut Split<'_, &str>,
            s: &str,
        ) -> Result<(isize, isize), anyhow::Error> {
            let dim = split
                .next()
                .ok_or(anyhow!("failed to get dimension spec: {}", s))?;
            let dim = dim
                .strip_prefix("x=")
                .or(dim.strip_prefix("y="))
                .or(dim.strip_prefix("z="))
                .ok_or(anyhow!("failed to strip dim prefix: {}", s))?;

            let mut fst_snd = dim.split("..");
            let fst = fst_snd
                .next()
                .unwrap()
                .parse()
                .map_err(Into::<anyhow::Error>::into)?;
            let snd = fst_snd
                .next()
                .unwrap()
                .parse()
                .map_err(Into::<anyhow::Error>::into)?;

            Ok((fst, snd))
        }

        let mut instruction_cuboid = s.split(" ");
        let instruction = instruction_cuboid
            .next()
            .ok_or_else(|| anyhow!("Failed to find instruction: {}", s))?;
        let cuboid = instruction_cuboid
            .next()
            .ok_or_else(|| anyhow!("Failed to find cuboid: {}", s))?;

        let instruction = match instruction {
            "on" => RebootInstruction::On,
            "off" => RebootInstruction::Off,
            _ => {
                return Err(anyhow!(
                    "Failed to parse instruction, expected on/off, got: {} in {}",
                    instruction,
                    s
                ));
            }
        };

        let mut cuboid_dims = cuboid.split(",");
        let (x_f, x_e) = parse_dim(&mut cuboid_dims, s)?;
        let (y_f, y_e) = parse_dim(&mut cuboid_dims, s)?;
        let (z_f, z_e) = parse_dim(&mut cuboid_dims, s)?;

        Ok(Self {
            x0: x_f,
            x1: x_e,
            y0: y_f,
            y1: y_e,
            z0: z_f,
            z1: z_e,
            instruction,
        })
    }
}

impl RebootManual {
    fn on_cubes_count(&self, limit: Option<(isize, isize)>) -> isize {
        if limit.is_some() {
            let limit_cuboids: Vec<_> = self
                .0
                .iter()
                .cloned()
                .filter(|c| c.in_limit(&limit))
                .collect();

            return Self(limit_cuboids).on_cubes_count(None);
        }

        let mut matching_cubes: Vec<RebootCuboid> = vec![];

        for cube in self.0.iter() {
            let mut new_intersects = vec![];
            for other in matching_cubes.iter() {
                if let Some(c) = other.intersect(&cube) {
                    new_intersects.push(c);
                }
            }

            if let RebootInstruction::On = cube.instruction {
                matching_cubes.push(cube.clone());
            }

            matching_cubes.extend(new_intersects);
        }

        matching_cubes.iter().map(RebootCuboid::area).sum()
    }
}

#[derive(Debug)]
struct RebootManual(Vec<RebootCuboid>);

impl FromStr for RebootManual {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.lines().map(str::parse).collect::<Result<_, _>>()?))
    }
}

fn main() -> anyhow::Result<()> {
    let reboot_manual: RebootManual = fs::read_to_string("./input")?.parse()?;

    println!(
        "Total of {} cubes are on (limited to +/- 50 dimensions)",
        reboot_manual.on_cubes_count(Some((-50, 50)))
    );
    println!(
        "Total of {} cubes are on (unlimited)",
        reboot_manual.on_cubes_count(None)
    );
    Ok(())
}

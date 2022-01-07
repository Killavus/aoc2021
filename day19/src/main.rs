use nalgebra as na;
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    error::Error,
    fs,
    iter::Peekable,
    str::FromStr,
};

const ROTATIONS: [na::Matrix3<i64>; 24] = [
    na::matrix![1, 0, 0;
                0, 1, 0;
                0, 0, 1],
    na::matrix![1, 0, 0;
                0, 0, -1;
                0, 1, 0],
    na::matrix![1, 0, 0;
                0, -1, 0;
                0, 0, -1],
    na::matrix![1, 0, 0;
                0, 0, 1;
                0, -1, 0],
    na::matrix![0, -1, 0;
                1, 0, 0;
                0, 0, 1],
    na::matrix![0, 0, 1;
                1, 0, 0;
                0, 1, 0],
    na::matrix![0, 1, 0;
                1, 0, 0;
                0, 0, -1],
    na::matrix![0, 0, -1;
                1, 0, 0;
                0, -1, 0],
    na::matrix![-1, 0, 0;
                0, -1, 0;
                0, 0, 1],
    na::matrix![-1, 0, 0;
                0, 0, -1;
                0, -1, 0],
    na::matrix![-1, 0, 0;
                0, 1, 0;
                0, 0, -1],
    na::matrix![-1, 0, 0;
                0, 0, 1;
                0, 1, 0],
    na::matrix![0, 1, 0;
                -1, 0, 0;
                0, 0, 1],
    na::matrix![0, 0, 1;
                -1, 0, 0;
                0, -1, 0],
    na::matrix![0, -1, 0;
                -1, 0, 0;
                0, 0, -1],
    na::matrix![0, 0, -1;
                -1, 0, 0;
                0, 1, 0],
    na::matrix![0, 0, -1;
                0, 1, 0;
                1, 0, 0],
    na::matrix![0, 1, 0;
                0, 0, 1;
                1, 0, 0],
    na::matrix![0, 0, 1;
                0, -1, 0;
                1, 0, 0],
    na::matrix![0, -1, 0;
                0, 0, -1;
                1, 0, 0],
    na::matrix![0, 0, -1;
                0, -1, 0;
                -1, 0, 0],
    na::matrix![0, -1, 0;
                0, 0, 1;
                -1, 0, 0],
    na::matrix![0, 0, 1;
                0, 1, 0;
                -1, 0, 0],
    na::matrix![0, 1, 0;
                0, 0, -1;
                -1, 0, 0],
];

#[derive(Debug)]
struct Scanner {
    beacons: HashSet<na::Point3<i64>>,
    id: usize,
}

#[derive(Debug)]
struct ScannerMap(Vec<Scanner>);

impl Scanner {
    fn from_iter<'line, I>(mut iter: Peekable<I>) -> (Self, Peekable<I>)
    where
        I: Iterator<Item = &'line str>,
    {
        let id = iter
            .next()
            .expect("missing header")
            .strip_prefix("--- scanner ")
            .and_then(|s| s.strip_suffix(" ---"))
            .expect("invalid format for header")
            .parse()
            .expect("failed to parse id");

        let mut points = HashSet::new();

        loop {
            let xyz = iter
                .next()
                .expect("failed to get beacon")
                .split(',')
                .flat_map(|n| n.parse())
                .collect::<Vec<i64>>();

            let mut point = na::Point3::origin();
            point[0] = xyz[0];
            point[1] = xyz[1];
            point[2] = xyz[2];

            points.insert(point);

            match iter.peek() {
                Some(next_line) => {
                    if next_line.is_empty() {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }

        (
            Self {
                id,
                beacons: points,
            },
            iter,
        )
    }
}

impl FromStr for Scanner {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_iter(s.lines().peekable()).0)
    }
}

impl FromStr for ScannerMap {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.lines().peekable();

        let mut scanners = vec![];
        loop {
            let (scanner, iter_new) = Scanner::from_iter(iter);
            scanners.push(scanner);
            iter = iter_new;

            match iter.peek() {
                Some(_) => {
                    iter.next();
                }
                None => {
                    break;
                }
            }
        }

        Ok(Self(scanners))
    }
}

fn point_cloud_distances(
    cloud: &HashSet<na::Point3<i64>>,
) -> HashMap<na::Point3<i64>, HashSet<i64>> {
    let mut result = HashMap::new();
    for beacon in cloud.iter() {
        let mut distances = HashSet::new();
        for beacon2 in cloud.iter() {
            let distance = (beacon - beacon2).abs().sum();
            distances.insert(distance);
        }

        result.insert(beacon.to_owned(), distances);
    }

    result
}

type DistanceMap = HashMap<na::Point3<i64>, HashSet<i64>>;

fn find_match(
    origin_dmap: &DistanceMap,
    unknown_dmap: &DistanceMap,
) -> Option<(na::Point3<i64>, na::Point3<i64>)> {
    for (orig_point, orig_distances) in origin_dmap.iter() {
        for (unk_point, unk_distances) in unknown_dmap.iter() {
            if orig_distances.intersection(unk_distances).count() >= 12 {
                return Some((orig_point.to_owned(), unk_point.to_owned()));
            }
        }
    }

    None
}

impl Scanner {
    fn distances(&self) -> HashMap<na::Point3<i64>, HashSet<i64>> {
        point_cloud_distances(&self.beacons)
    }
}

fn scanner_positions(scanners: &[Scanner]) -> (Vec<na::Point3<i64>>, HashSet<na::Point3<i64>>) {
    let mut origins: Vec<na::Point3<i64>> = vec![na::Point3::origin()];
    let mut ids = vec![scanners[0].id];
    let mut known_cloud: HashSet<na::Point3<i64>> = HashSet::new();
    known_cloud.extend(&scanners[0].beacons);

    while origins.len() < scanners.len() {
        let orig_distances = point_cloud_distances(&known_cloud);

        for unknown in scanners.iter() {
            if ids.contains(&unknown.id) {
                continue;
            }

            if let Some((orig_point, unk_point)) = find_match(&orig_distances, &unknown.distances())
            {
                for matrix in ROTATIONS {
                    let unk_point = matrix * unk_point;
                    let translation_v = orig_point - unk_point;

                    let translated_points = unknown
                        .beacons
                        .iter()
                        .cloned()
                        .map(|point| matrix * point + translation_v)
                        .collect::<HashSet<_>>();

                    if translated_points.intersection(&known_cloud).count() >= 12 {
                        known_cloud.extend(translated_points.into_iter());
                        origins.push((-translation_v).into());
                        ids.push(unknown.id);
                        break;
                    }
                }
            }
        }
    }

    (origins, known_cloud)
}

fn main() -> Result<(), Box<dyn Error>> {
    let scanner_map: ScannerMap = fs::read_to_string("./input")?.parse()?;

    let (origins, points) = scanner_positions(&scanner_map.0);

    println!("There are {} unique points seen by scanners", points.len());

    let mut distances = vec![];
    for origin in origins.iter() {
        for origin2 in origins.iter() {
            distances.push((origin - origin2).abs().sum());
        }
    }

    println!(
        "Scanners are at most {} units apart",
        distances.iter().max().unwrap()
    );

    Ok(())
}

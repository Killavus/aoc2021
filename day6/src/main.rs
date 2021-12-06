use std::fs;

fn simulate_growth(lanternfishes: &[usize], days: usize) -> usize {
    let mut histogram: [usize; 9] = [0; 9];
    lanternfishes.iter().copied().for_each(|fish_age| {
        histogram[fish_age] += 1;
    });

    let mut day_no = 1;
    while day_no <= days {
        let mut new_histogram = [0; 9];

        let new_lanternfish_count = histogram[0];

        for idx in 1..=8 {
            new_histogram[idx - 1] = histogram[idx];
        }

        new_histogram[8] += new_lanternfish_count;
        new_histogram[6] += new_lanternfish_count;
        histogram = new_histogram;
        day_no += 1;
    }

    histogram.into_iter().sum()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lanternfishes: Vec<usize> = fs::read_to_string("./input")?
        .lines()
        .flat_map(|line| line.split(","))
        .flat_map(str::parse)
        .collect();

    println!(
        "Number of lanternfishes after 80 days: {}",
        simulate_growth(&lanternfishes, 80)
    );

    println!(
        "Number of lanternfishes after 256 days: {}",
        simulate_growth(&lanternfishes, 256)
    );

    Ok(())
}

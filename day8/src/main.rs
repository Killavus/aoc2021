use anyhow::Result;
use notepad::NoteEntry;
use std::fs;

mod notepad;

fn main() -> Result<()> {
    let note_entries: Vec<NoteEntry> = fs::read_to_string("./input")?
        .lines()
        .flat_map(str::parse)
        .collect();

    println!(
        "Number of appearances of 1, 4, 7, 8 in output values: {}",
        note_entries
            .iter()
            .map(NoteEntry::unique_segments_digits_count)
            .sum::<usize>()
    );

    println!(
        "Sum of all output values in note entries: {}",
        note_entries
            .iter()
            .map(NoteEntry::unscrambled_output_value)
            .sum::<usize>()
    );

    Ok(())
}

pub fn capitalize(s: &str) -> String {
    s.chars()
        .take(1)
        .flat_map(|c| c.to_uppercase())
        .chain(s.chars().skip(1))
        .collect()
}

pub mod specialization_constants;
pub mod entry_points;

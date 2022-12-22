use anyhow::{anyhow, Result};

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let mut out = String::new();
    anyhow::ensure!(input.read_line(&mut out)? > 0, "No input data");
    out.truncate(out.trim_end().len());

    Ok(out.chars().collect())
}

/// Find the number of characters processed before a unique sequence of length `marker_len`
fn find_marker(s: &[char], marker_len: usize) -> Option<usize> {
    s.windows(marker_len)
     .position(|window| {
         window.iter().cloned()
               .map(|c| 1 << (c as u8 - b'a'))
               .sum::<u32>()
               .count_ones() == (marker_len as u32)
     })
     .map(|pos| pos + marker_len) // offset for length of marker
}

fn solve1(s: &Input) -> Result<usize> {
     find_marker(s, 4).ok_or_else(|| anyhow!("No matching position"))
}

fn solve2(s: &Input) -> Result<usize> {
     find_marker(s, 14).ok_or_else(|| anyhow!("No matching position"))
}

problem!(load_input => Vec<char> => (solve1, solve2));

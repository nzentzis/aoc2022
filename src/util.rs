use anyhow::Result;
use regex::{Regex, Captures};

pub fn read_lines<F: FnMut(&str) -> Result<T>, T>(
    input: &mut dyn std::io::BufRead,
    mut parser: F
) -> Result<Vec<T>> {
    let mut out = Vec::new();
    let mut line = String::new();
    while input.read_line(&mut line)? > 0 {
        let trimmed = line.trim();

        if !trimmed.is_empty() {
            out.push((parser)(trimmed)?);
        }
        line.clear();
    }

    Ok(out)
}

pub fn read_lines_regex<F: FnMut(Captures) -> Result<T>, T>(
    input: &mut dyn std::io::BufRead,
    expr: &str,
    mut parser: F
) -> Result<Vec<T>> {
    let expr = Regex::new(expr)?;
    let mut lines = 0;
    read_lines(input, |s| {
        lines += 1;
        let m = expr.captures(s)
                    .ok_or_else(|| anyhow::anyhow!("No regex match on line {}", lines))?;
        (parser)(m)
    })
}

/// Load and parse lines of the file
pub fn load_lines<T: std::str::FromStr>(
    input: &mut dyn std::io::BufRead
) -> Result<Vec<T>>
where anyhow::Error: From<T::Err>,
{
    read_lines(input, |line| T::from_str(line).map_err(|e| e.into()))
}

/// Load a grid containing data in each character
///
/// Width and height of the grid are set automatically based on the input file.
pub fn load_grid<T: TryFrom<char>>(
    input: &mut dyn std::io::BufRead
) -> Result<crate::grid::Grid<T>>
where T::Error: Send+Sync,
      anyhow::Error: From<T::Error>,
{
    use crate::grid::Grid;

    let mut data = Vec::new();
    let mut width = None;

    let mut line = String::new();
    while input.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match width.as_ref() {
            Some(w) => {
                anyhow::ensure!(*w == trimmed.len(), "Grid rows are not allowed to vary in width");
            }
            None => {
                width = Some(trimmed.len());
            }
        }

        // parse this line
        for c in trimmed.chars() {
            data.push(T::try_from(c)?);
        }

        line.clear();
    }

    Ok(Grid::from_data(data, width.unwrap_or(0)))
}

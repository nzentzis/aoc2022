use anyhow::Result;

use std::ops::Deref;

use crate::grid::Grid;

struct Problem {
    grid: Grid<u8>,
    start: (usize, usize),
    end: (usize, usize),
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let grid = crate::util::load_grid::<char>(input)?;
    
    let start = grid.find('S').next()
               .ok_or_else(|| anyhow::anyhow!("No start position on grid"))?
               .coords();
    let end = grid.find('E').next()
             .ok_or_else(|| anyhow::anyhow!("No start position on grid"))?
             .coords();

    let grid = grid.map(|c| match *c {
        'S' => 0,
        'E' => b'z' - b'a',
        c => (c as u8) - b'a',
    });

    Ok(Problem { grid, start, end })
}

/// Compute how many steps are required to reach the endpoint from any position
fn steps_into(input: &Input) -> Grid<usize> {
    let mut steps = Grid::filled_like(&input.grid, usize::MAX - 1);
    steps.set(input.end, 0);

    let mut changed = true;
    while changed {
        changed = false;
        for point in input.grid.points() {
            let min_height = point.deref().saturating_sub(1);
            let l_steps = *steps.get(point.coords()) + 1;

            if let Some(left) = point.left() {
                let s = steps.get_mut(left.coords());
                if *left >= min_height && *s > l_steps {
                    changed = true;
                    *s = l_steps;
                }
            }
            if let Some(right) = point.right() {
                let s = steps.get_mut(right.coords());
                if *right >= min_height && *s > l_steps {
                    changed = true;
                    *s = l_steps;
                }
            }
            if let Some(up) = point.up() {
                let s = steps.get_mut(up.coords());
                if *up >= min_height && *s > l_steps {
                    changed = true;
                    *s = l_steps;
                }
            }
            if let Some(down) = point.down() {
                let s = steps.get_mut(down.coords());
                if *down >= min_height && *s > l_steps {
                    changed = true;
                    *s = l_steps;
                }
            }
        }
    }

    steps
}

fn solve1(input: &Input) -> Result<usize> {
    Ok(*steps_into(input).get(input.start))
}

fn solve2(input: &Input) -> Result<usize> {
    // we're computing the inverse here - steps into the end point from anywhere
    let steps = steps_into(input);
    input.grid.find(0)
              .map(|p| *steps.get(p.coords()))
              .min()
              .ok_or_else(|| anyhow::anyhow!("Empty grid"))
}

problem!(load_input => Problem => (solve1, solve2));

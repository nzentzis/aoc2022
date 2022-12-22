use anyhow::Result;

use crate::grid::{Grid, GridPoint};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
struct Tree(u8);

impl TryFrom<char> for Tree {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Self> {
        anyhow::ensure!(c.is_ascii_digit(), "Invalid tree height");
        Ok(Self((c as u8) - b'0'))
    }
}

fn solve1(input: &Input) -> Result<usize> {
    let mut visible = Grid::filled_like(input, false);

    /// A single visibility pass
    ///
    /// Update the visibility status (the first tuple elem) of each element based on the heights
    /// (the second tuple elem).
    fn pass<'a, I: Iterator<Item=(&'a mut bool, &'a Tree)>>(iter: I) {
        let mut current = None; // highest previously-seen height
        for (visible, height) in iter {
            match current.as_mut() {
                None => {
                    *visible = true;
                    current = Some(height.0);
                }
                Some(cur) if height.0 > *cur => {
                    *visible = true;
                    *cur = height.0;
                }
                _ => {}
            }
        }
    }

    // rows
    for row in 0..input.height() {
        pass(visible.row_iter_mut(row).zip(input.row_iter(row))); // downwards
        pass(visible.row_iter_mut(row).rev().zip(input.row_iter(row).rev())); // upwards
    }

    // columns
    for col in 0..input.width() {
        pass(visible.col_iter_mut(col).zip(input.col_iter(col))); // right
        pass(visible.col_iter_mut(col).rev().zip(input.col_iter(col).rev())); // left
    }

    Ok(visible.into_cells().filter(|b| *b).count())
}

fn solve2(input: &Input) -> Result<usize> {
    fn visibility<'g, I: Iterator<Item=GridPoint<'g, Tree>>>(iter: I, current: Tree) -> usize {
        let mut out = 0;
        for t in iter {
            out += 1;
            if t.0 >= current.0 {
                break;
            }
        }

        out
    }

    fn score(tree: GridPoint<Tree>) -> usize {
        let l_dist = visibility(tree.walk_left(), *tree);
        let r_dist = visibility(tree.walk_right(), *tree);
        let u_dist = visibility(tree.walk_up(), *tree);
        let d_dist = visibility(tree.walk_down(), *tree);

        l_dist * r_dist * u_dist * d_dist
    }

    input.points().map(score).max().ok_or_else(|| anyhow::anyhow!("No points on input grid"))
}

problem!(crate::util::load_grid => Grid<Tree> => (solve1, solve2));

use anyhow::Result;

use crate::grid::Grid;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Cell {
    Empty,
    Wall,
    Sand
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let paths = crate::util::read_lines(input, |line| {
        line.split(" -> ")
            .map(|p| {
                let (x,y) = p.split_once(',')
                             .ok_or_else(|| anyhow::anyhow!("Invalid coordinates"))?;
                Ok((x.parse::<usize>()?, y.parse::<usize>()?))
            })
            .collect::<Result<Vec<_>>>()
    })?;

    let width = paths.iter().map(|p| p.iter().map(|t| t.0).max().unwrap()).max().unwrap() + 200;
    let height = paths.iter().map(|p| p.iter().map(|t| t.1).max().unwrap()).max().unwrap() + 2;

    let mut cells = Grid::filled(width, height, Cell::Empty);
    for p in paths {
        let mut pos = p[0];
        for next in &p[1..] {
            if pos.0 < next.0 {
                assert_eq!(pos.1, next.1);
                for x in pos.0..=next.0 {
                    cells.set((x, pos.1), Cell::Wall);
                }
            } else if pos.0 > next.0 {
                assert_eq!(pos.1, next.1);
                for x in next.0..=pos.0 {
                    cells.set((x, pos.1), Cell::Wall);
                }
            } else if pos.1 < next.1 {
                assert_eq!(pos.0, next.0);
                for y in pos.1..=next.1 {
                    cells.set((pos.0, y), Cell::Wall);
                }
            } else if pos.1 > next.1 {
                assert_eq!(pos.0, next.0);
                for y in next.1..=pos.1 {
                    cells.set((pos.0, y), Cell::Wall);
                }
            }

            pos = *next;
        }
    }

    Ok(Problem { cells })
}

struct Problem {
    cells: Grid<Cell>,
}

/// Compute the landing point of the next dropped sand unit
///
/// If the sand will never land, return `None`.
fn landing_point(state: &Grid<Cell>) -> Result<(usize, usize), (usize, usize)> {
    let mut pos = (500, 0);

    while state.try_get(pos).is_some() {
        // try down
        let Some(below) = state.try_get((pos.0, pos.1+1)) else { return Err(pos); };
        if *below == Cell::Empty {
            pos.1 += 1;
            continue;
        }

        // try left
        let Some(below) = state.try_get((pos.0-1, pos.1+1)) else { unreachable!() };
        if *below == Cell::Empty {
            pos.1 += 1;
            pos.0 -= 1;
            continue;
        }

        let Some(below) = state.try_get((pos.0+1, pos.1+1)) else { unreachable!() };
        if *below == Cell::Empty {
            pos.1 += 1;
            pos.0 += 1;
            continue;
        }

        return Ok(pos);
    }

    unreachable!()
}

fn solve1(input: &Input) -> Result<usize> {
    let mut state = input.cells.clone();
    while let Ok(pos) = landing_point(&state) {
        let c = state.get_mut(pos);
        assert_eq!(*c, Cell::Empty);
        *c = Cell::Sand;
    }

    Ok(state.cells().filter(|c| **c == Cell::Sand).count())
}

fn solve2(input: &Input) -> Result<usize> {
    let mut state = input.cells.clone();
    while *state.get((500, 0)) == Cell::Empty {
        match landing_point(&state) {
            Ok(pos) => {
                let c = state.get_mut(pos);
                assert_eq!(*c, Cell::Empty);
                *c = Cell::Sand;
            }
            Err(pos) => {
                let c = state.get_mut(pos);
                assert_eq!(*c, Cell::Empty);
                *c = Cell::Sand;
            }
        }
    }

    Ok(state.cells().filter(|c| **c == Cell::Sand).count())
}

problem!(load_input => Problem => (solve1, solve2));

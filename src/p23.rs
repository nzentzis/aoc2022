use anyhow::Result;

use crate::grid::Grid;

#[derive(Copy, Clone, PartialEq, Eq)]
enum Cell {
    Empty,
    Elf,
}

impl TryFrom<char> for Cell {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Self> {
        match c {
            '#' => Ok(Self::Elf),
            '.' => Ok(Self::Empty),
            _ => Err(anyhow::anyhow!("Invalid cell")),
        }
    }
}

type Coords = (isize, isize);

#[derive(Copy, Clone)]
enum Action {
    Up,
    Down,
    Left,
    Right
}

impl Action {
    const DIRECTIONS: &[Action] = &[
        Action::Up, Action::Down, Action::Left, Action::Right
    ];

    fn feasible(
        &self,
        (x, y): (isize, isize),
        presence: &fnv::FnvHashSet<(isize, isize)>
    ) -> bool {
        let (a, b, c) = match self {
            Self::Up    => ((x-1, y-1), (x, y-1), (x+1, y-1)),
            Self::Down  => ((x-1, y+1), (x, y+1), (x+1, y+1)),
            Self::Left  => ((x-1, y+1), (x-1, y), (x-1, y-1)),
            Self::Right => ((x+1, y+1), (x+1, y), (x+1, y-1)),
        };
        !(presence.contains(&a) || presence.contains(&b) || presence.contains(&c))
    }

    fn target(&self, (x, y): (isize, isize)) -> (isize, isize) {
        match self {
            Self::Up    => (x, y-1),
            Self::Down  => (x, y+1),
            Self::Left  => (x-1, y),
            Self::Right => (x+1, y),
        }
    }
}

fn neighbors((x, y): (isize, isize), presence: &fnv::FnvHashSet<(isize, isize)>) -> usize {
    presence.contains(&(x-1, y-1)) as usize +
    presence.contains(&(x  , y-1)) as usize +
    presence.contains(&(x+1, y-1)) as usize +
    presence.contains(&(x-1, y  )) as usize +
    presence.contains(&(x+1, y  )) as usize +
    presence.contains(&(x-1, y+1)) as usize +
    presence.contains(&(x  , y+1)) as usize +
    presence.contains(&(x+1, y+1)) as usize
}

struct Simulation {
    elves: Vec<Coords>,
    presence: fnv::FnvHashSet<Coords>,
    proposed: fnv::FnvHashMap<Coords, Action>,
    collider: fnv::FnvHashMap<Coords, u8>,
    order: usize, // start offset into [NSWE] array
}

impl Simulation {
    fn new(grid: &Grid<Cell>) -> Self {
        let elves = grid.points()
                   .filter(|p| **p == Cell::Elf)
                   .map(|p| p.coords())
                   .map(|(x, y)| (x as isize, y as isize))
                   .collect::<Vec<_>>();
        let presence = fnv::FnvHashSet::default();
        let proposed = fnv::FnvHashMap::default();
        let collider = fnv::FnvHashMap::default();

        Self { elves, presence, proposed, collider, order: 0 }
    }

    fn tick(&mut self) {
        // first half
        self.presence.clear();
        self.presence.extend(self.elves.iter().cloned());

        self.proposed.clear();
        self.proposed.extend(self.elves.iter().cloned()
                            .filter(|elf| neighbors(*elf, &self.presence) != 0)
                            .filter_map(|elf| {
                                // choose direction
                                for i in 0..4 {
                                    let offs = (self.order + i) % 4;
                                    let dir = Action::DIRECTIONS[offs];
                                    if dir.feasible(elf, &self.presence) {
                                        return Some((elf, dir));
                                    }
                                }

                                None
                            }));

        // update collision logic
        self.collider.clear();
        for e in &self.elves {
            if let Some(dir) = self.proposed.get(e) {
                *self.collider.entry(dir.target(*e)).or_insert(0) += 1;
            }
        }

        // second half
        for e in self.elves.iter_mut() {
            if let Some(dir) = self.proposed.get(e) {
                let tgt = dir.target(*e);
                if self.collider.get(&tgt) == Some(&1) {
                    *e = tgt;
                }
            }
        }

        // direction updates
        self.order = (self.order + 1) % 4;
    }

    fn bounds(&self) -> (usize, usize) {
        let min_x = self.elves.iter().map(|e| e.0).min().unwrap();
        let max_x = self.elves.iter().map(|e| e.0).max().unwrap();
        let min_y = self.elves.iter().map(|e| e.1).min().unwrap();
        let max_y = self.elves.iter().map(|e| e.1).max().unwrap();

        ((max_x - min_x + 1) as usize,
         (max_y - min_y + 1) as usize)
    }
}

fn solve1(input: &Input) -> Result<usize> {
    let elves = input.points().filter(|p| **p == Cell::Elf).count();

    let mut sim = Simulation::new(input);
    for _ in 0..10 {
        sim.tick();
    }
    let bounds = sim.bounds();

    Ok((bounds.0 * bounds.1) - elves)
}

fn solve2(input: &Input) -> Result<usize> {
    let mut round = 1;
    let mut sim = Simulation::new(input);
    let mut last_elves = sim.elves.clone();
    sim.tick();
    while last_elves != sim.elves {
        round += 1;
        last_elves.copy_from_slice(&sim.elves[..]);
        sim.tick();
    }

    Ok(round)
}

problem!(crate::util::load_grid => Grid<Cell> => (solve1, solve2));

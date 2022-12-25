use anyhow::Result;

use crate::grid::{Grid, GridPoint};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Cell {
    Unset,
    Space,
    Wall,
}

impl TryFrom<char> for Cell {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            '_' => Ok(Self::Unset),
            '.' => Ok(Self::Space),
            '#' => Ok(Self::Wall),
            _   => Err(anyhow::anyhow!("Invalid cell value"))
        }
    }
}

impl Cell {
    fn is_set(self) -> bool {
        self != Self::Unset
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Dir { Left, Right, Up, Down }

impl Dir {
    fn turn_left(self) -> Self {
        match self {
            Self::Left  => Self::Down,
            Self::Right => Self::Up,
            Self::Up    => Self::Left,
            Self::Down  => Self::Right,
        }
    }

    fn turn_right(self) -> Self {
        match self {
            Self::Left  => Self::Up,
            Self::Right => Self::Down,
            Self::Up    => Self::Right,
            Self::Down  => Self::Left,
        }
    }

    fn as_num(&self) -> usize {
        match self {
            Self::Right => 0,
            Self::Down  => 1,
            Self::Left  => 2,
            Self::Up    => 3,
        }
    }

    fn flip(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Copy, Clone)]
enum Move {
    Left,
    Right,
    Move(usize),
}

struct Problem {
    map: Grid<Cell>,

    directions: Vec<Move>,
}

fn load_input(input: &mut dyn std::io::BufRead) -> Result<Input> {
    let mut grid_lines = Vec::new();
    let mut grid_data = String::new();
    while input.read_line(&mut grid_data)? > 1 {
        let mut s = std::mem::take(&mut grid_data);
        s.truncate(s.trim_end().len());
        grid_lines.push(s);
    }

    let grid_width = grid_lines.iter().map(|s| s.len()).max().unwrap();
    for s in grid_lines.iter_mut() {
        for _ in s.len()..grid_width {
            s.push(' ');
        }
        *s = s.replace(' ', "_");
    }

    let grid_data = grid_lines.join("\n");

    let mut dirs_line = String::new();
    anyhow::ensure!(input.read_line(&mut dirs_line)? > 1, "No directions line");

    let mut grid_cursor = std::io::Cursor::new(&grid_data);
    let map = crate::util::load_grid(&mut grid_cursor)?;

    let mut directions = Vec::new();
    let mut accum = String::new();
    for ch in dirs_line.chars() {
        match ch {
            '0'..='9' => {
                accum.push(ch);
            }
            'L' => {
                if !accum.is_empty() {
                    directions.push(Move::Move(accum.parse()?));
                    accum.clear();
                }
                directions.push(Move::Left);
            }
            'R' => {
                if !accum.is_empty() {
                    directions.push(Move::Move(accum.parse()?));
                    accum.clear();
                }
                directions.push(Move::Right);
            }
            '\n' => {}
            c => anyhow::bail!("Invalid path character: {}", c)
        }
    }
    if !accum.is_empty() {
        directions.push(Move::Move(accum.parse()?));
        accum.clear();
    }

    Ok(Problem { map, directions })
}

struct Actor<'g> {
    pos: GridPoint<'g, Cell>,
    dir: Dir,
}

impl<'g> Actor<'g> {
    fn new(map: &'g Grid<Cell>) -> Self {
        // find start point
        let mut pos = map.point((0, 0));
        while !(*pos == Cell::Space) {
            pos = pos.right().unwrap();
        }

        Self {
            pos,
            dir: Dir::Right,
        }
    }

    /// Get the cell in front of the actor
    fn forward<W: Fn(&'g Problem, GridPoint<'g, Cell>, Dir) -> (GridPoint<'g, Cell>, Dir)>(
        &self,
        map: &'g Problem,
        wrap: W,
    ) -> (GridPoint<'g, Cell>, Dir) {
        let c = match self.dir {
            Dir::Left => {
                self.pos.left()
            },
            Dir::Right => {
                self.pos.right()
            },
            Dir::Up => {
                self.pos.up()
            },
            Dir::Down => {
                self.pos.down()
            },
        };

        if c.map(|cell| *cell == Cell::Unset).unwrap_or(true) {
            (wrap)(map, self.pos, self.dir)
        } else {
            (c.unwrap(), self.dir)
        }
    }

    fn execute<W: Fn(&'g Problem, GridPoint<'g, Cell>, Dir) -> (GridPoint<'g, Cell>, Dir)>(
        &mut self,
        map: &'g Problem,
        insn: &Move,
        wrap: W,
    ) {
        match insn {
            Move::Left => {
                self.dir = self.dir.turn_left();
            }
            Move::Right => {
                self.dir = self.dir.turn_right();
            }
            Move::Move(n) => {
                for _ in 0..*n {
                    let (new_pos, new_dir) = self.forward(map, &wrap);
                    if *new_pos == Cell::Wall {
                        break;
                    }
                    self.pos = new_pos;
                    self.dir = new_dir;
                }
            }
        }
    }

    /// Compute the code based on the actor's current state
    fn code(&self) -> usize {
        let (col, row) = self.pos.coords();
        let dir = self.dir.as_num();
        let col = col + 1; // offset by one due to zero vs one-based indexing
        let row = row + 1;
        1000*row + 4*col + dir
    }
}

fn solve1(input: &Input) -> Result<usize> {
    let row_wrap = (0..input.map.height()).into_iter()
                  .map(|y| {
                      let first = input.map.row_iter(y).position(|x| *x != Cell::Unset).unwrap();
                      let last = input.map.row_iter(y).rposition(|x| *x != Cell::Unset).unwrap();
                      (first, last)
                  }).collect::<Vec<_>>();
    let col_wrap = (0..input.map.width()).into_iter()
                  .map(|x| {
                      let first = input.map.col_iter(x).position(|x| *x != Cell::Unset).unwrap();
                      let last = input.map.col_iter(x).rposition(|x| *x != Cell::Unset).unwrap();
                      (first, last)
                  }).collect::<Vec<_>>();

    let mut actor = Actor::new(&input.map);
    for m in &input.directions {
        actor.execute(input, m, |_, pos, dir| {
            // wrap around
            let (col, row) = pos.coords();
            let out = match dir {
                Dir::Left => input.map.point((row_wrap[row].1, row)),
                Dir::Right => input.map.point((row_wrap[row].0, row)),
                Dir::Up => input.map.point((col, col_wrap[col].1)),
                Dir::Down => input.map.point((col, col_wrap[col].0)),
            };

            (out, dir)
        });
    }

    Ok(actor.code())
}

/// A mapping of a 2-dimensional input grid onto a 3-dimensional cube
///
/// This contains information about six square regions of the input grid. Each region represents a
/// face on the cube, and stores which vertex of the logical cube is adjacent to each face corner.
///
/// Cube vertices are numbered as follows:
///
/// ```text
///     6-----------7
///    /|          /|
///   / |         / |
///  /  |        /  |
/// 4-----------5   |
/// |   2-------|---3
/// |  /        |  /
/// | /         | /
/// |/          |/
/// 0-----------1
/// ```
///
/// Faces are numbered as follows:
///
///  | Vertices | Face number | Diagram position |
///  |----------|-------------|------------------|
///  | 0123     | 0           | Bottom           |
///  | 0264     | 1           | Left             |
///  | 0154     | 2           | Near             |
///  | 1573     | 3           | Right            |
///  | 2673     | 4           | Far              |
///  | 4675     | 5           | Top              |
struct CubeMapping {
    /// Faces of the cube, in logical face order (i.e. the index is the face number)
    faces: Vec<Face>,
}

#[derive(Debug)]
struct Face {
    /// Vertex indices of this face
    verts: [usize; 4],

    /// Grid coordinates of the face's corner cells
    ///
    /// The coordinate at index `i` here corresponds to the corner bordering vertex `verts[i]`.
    coords: [(usize, usize); 4],

    /// Min and max grid X-coordinate
    x_bounds: (usize, usize),

    /// Min and max grid Y-coordinate
    y_bounds: (usize, usize),
}

impl Face {
    fn new(verts: [usize; 4], coords: [(usize, usize); 4]) -> Self {
        let x_min = coords.iter().cloned().map(|c| c.0).min().unwrap();
        let x_max = coords.iter().cloned().map(|c| c.0).max().unwrap();

        let y_min = coords.iter().cloned().map(|c| c.1).min().unwrap();
        let y_max = coords.iter().cloned().map(|c| c.1).max().unwrap();

        Self {
            verts, coords,
            x_bounds: (x_min, x_max),
            y_bounds: (y_min, y_max),
        }
    }

    fn contains_pos(&self, pos: (usize, usize)) -> bool {
        pos.0 >= self.x_bounds.0 && pos.0 <= self.x_bounds.1 &&
            pos.1 >= self.y_bounds.0 && pos.1 <= self.y_bounds.1
    }
}

impl CubeMapping {
    /// Given a populated input grid, compute how it maps onto a 3D cube
    ///
    /// This is used by the part 2 solution, and analyzes the unset-vs-set cells in the input grid to
    /// figure out how it maps onto a cube.
    ///
    /// The algorithm here has a few major stages:
    ///
    ///  1. Find the size of the cube
    ///  2. Find the faces and wrap them onto the logical cube
    fn new(grid: &Input) -> Self {
        // Find the size of the cube by finding the shortest contiguous row/column of set cells in
        // the input grid.
        //
        // For any unwrapping, at least one edge will be of length equal to that of a cube edge, so
        // this will find N for an NxNxN cube.
        let side_len = {
            fn min_run<'a, I: Iterator<Item=&'a Cell>>(xs: I) -> usize {
                let mut min_run = usize::MAX;
                let mut n = 0;
                for cell in xs {
                    if !cell.is_set() && n > 0 {
                        min_run = min_run.min(n);
                        n = 0;
                    } else if cell.is_set() {
                        n += 1;
                    }
                }
                if n > 0 {
                    min_run = min_run.min(n);
                }

                min_run
            }

            (0..grid.map.height()).into_iter().map(|y| min_run(grid.map.row_iter(y)))
                .chain((0..grid.map.width()).into_iter().map(|x| min_run(grid.map.col_iter(x))))
                .min()
                .unwrap()
        };

        #[derive(Copy, Clone)]
        /// A corner's corresponding location on the input grid
        struct GridCorner<'a> {
            pos: GridPoint<'a, Cell>,

            /// Whether the horizontal edge belonging to the same face travels along +X
            x_pos: bool,

            /// Whether the vertical edge belonging to the same face travels along +Y
            y_pos: bool,
        }

        impl GridCorner<'_> {
            fn follow_x(&self, n: usize) -> Self {
                let dx = if self.x_pos { n as isize - 1 } else { 1 - n as isize };
                Self {
                    pos: self.pos.offset((dx, 0)).unwrap(),
                    x_pos: !self.x_pos,
                    y_pos: self.y_pos,
                }
            }

            fn follow_y(&self, n: usize) -> Self {
                let dy = if self.y_pos { n as isize - 1 } else { 1 - n as isize };
                Self {
                    pos: self.pos.offset((0, dy)).unwrap(),
                    x_pos: self.x_pos,
                    y_pos: !self.y_pos,
                }
            }

            /// Flip along the X axis to where the connecting face would be
            fn mirror_x(&self) -> Option<Self> {
                let dx = if self.x_pos { -1 } else { 1 }; // go opposite our edge
                Some(Self {
                    pos: self.pos.offset((dx, 0))?,
                    x_pos: !self.x_pos,
                    y_pos: self.y_pos,
                })
            }

            /// Flip along the Y axis to where the connecting face would be
            fn mirror_y(&self) -> Option<Self> {
                let dy = if self.y_pos { -1 } else { 1 }; // go opposite our edge
                Some(Self {
                    pos: self.pos.offset((0, dy))?,
                    x_pos: self.x_pos,
                    y_pos: !self.y_pos,
                })
            }
        }

        // Map logical faces to sections of the input grid
        //
        // We do this by first mapping an arbitrary face to #0, and working outwards from there. As
        // each edge is fixed in place, look on the input grid for a corresponding face and map it
        // to the relevant set of opposite verts if it's there. Once all edges have been processed,
        // we'll have a complete mapping.
        let mut faces = vec![None, None, None, None, None, None];

        // Generate the seed face
        //
        // 1. Choose a set cell with two unset (or off-grid) cells neighboring it. This becomes our
        //    first face corner. This corner will be adjacent to vertex 0 on the output cube, and
        //    we use it as a starting point.
        // 2. Once we have an initial corner, exend two edges of length N and map them to vertices
        //    1 and 2 arbitrarily
        // 3. Finally, use the two perpendicular edges to complete the face and map a corner to
        //    vertex 3.
        let seed_corner = {
            grid.map.points()
                    .find_map(|p| {
                        let l = p.left().map(|c| c.is_set()).unwrap_or(false);
                        let r = p.right().map(|c| c.is_set()).unwrap_or(false);
                        let u = p.up().map(|c| c.is_set()).unwrap_or(false);
                        let d = p.down().map(|c| c.is_set()).unwrap_or(false);
                        let n = (l as u8) + (r as u8) + (u as u8) + (d as u8);

                        if n == 2 {
                            Some(GridCorner {
                                pos: p,
                                x_pos: r,
                                y_pos: d,
                            })
                        } else {
                            None
                        }
                    })
                    .expect("No corners on input grid")
        };

        struct PendingEntry<'a> {
            face: usize,
            edge: [usize; 2],
            corners: [GridCorner<'a>; 2],
        }
        let mut pending = Vec::new();

        {
            let a = seed_corner;
            let b = seed_corner.follow_x(side_len);
            let c = seed_corner.follow_y(side_len);
            let d = b.follow_y(side_len);
            faces[0] = Some(Face::new(
                [0, 1, 2, 3],
                [a.pos.coords(), b.pos.coords(), c.pos.coords(), d.pos.coords()]
            ));
            pending.extend([[(0, a), (1, b)],
                            [(0, a), (2, c)],
                            [(2, c), (3, d)],
                            [(3, d), (1, b)]].into_iter()
                          .map(|[(e0, c0), (e1, c1)]| PendingEntry {
                              face: 0,
                              edge: [e0, e1],
                              corners: [c0, c1],
                          }));
        }

        while let Some(entry) = pending.pop() {
            // 1. Are there valid grid cells opposite this edge?
            let is_horiz = entry.corners[0].x_pos != entry.corners[1].x_pos;

            // get corresponding grid corners on the other side of the edge
            let flipped = if is_horiz { entry.corners[0].mirror_y().zip(entry.corners[1].mirror_y()) }
                          else { entry.corners[0].mirror_x().zip(entry.corners[1].mirror_x()) };

            // if there's no valid flipped coords, or if the flipped cells are unset, skip this edge
            let Some((o_c0, o_c1)) = flipped else { continue };
            if !(o_c0.pos.is_set() && o_c1.pos.is_set()) {
                continue;
            }

            // 2. There's a valid grid face opposite this edge - is it already mapped?
            let opp_id = Self::opposite_face(entry.edge, entry.face);
            if faces[opp_id].is_some() {
                continue;
            }

            // 3. The opposite face is both available and unmapped. Convert new grid positions into
            //    face corners and map it.
            let o_c2 = if is_horiz { o_c0.follow_y(side_len) } else { o_c0.follow_x(side_len) };
            let o_c3 = if is_horiz { o_c1.follow_y(side_len) } else { o_c1.follow_x(side_len) };

            let a = (entry.edge[0], o_c0);
            let b = (entry.edge[1], o_c1);
            let c = (Self::opposite_vert(opp_id, entry.edge[0], entry.edge[1]), o_c2); // opposite A
            let d = (Self::opposite_vert(opp_id, entry.edge[1], entry.edge[0]), o_c3); // opposite B

            faces[opp_id] = Some(Face::new(
                [a.0, b.0, c.0, d.0],
                [a.1.pos.coords(), b.1.pos.coords(), c.1.pos.coords(), d.1.pos.coords()]
            ));
            pending.extend([(a, c), (b, d), (c, d)].into_iter()
                          .map(|((v0, c0), (v1, c1))| PendingEntry {
                              face: opp_id,
                              edge: [v0, v1],
                              corners: [c0, c1],
                          }));
        }

        Self { faces: faces.into_iter().map(|f| f.unwrap()).collect() }
    }

    /// Return the face index which shares `edge` with `face`
    ///
    /// # Panics
    /// This will panic if passed an invalid `(face, edge)` pair.
    fn opposite_face(edge: [usize; 2], face: usize) -> usize {
        let e0 = edge[0].min(edge[1]);
        let e1 = edge[0].max(edge[1]);

        match (face, e0, e1) {
            (0, 0, 1) => 2, (0, 0, 2) => 1, (0, 1, 3) => 3, (0, 2, 3) => 4, // bottom
            (1, 0, 2) => 0, (1, 0, 4) => 2, (1, 2, 6) => 4, (1, 4, 6) => 5, // left
            (2, 0, 1) => 0, (2, 1, 5) => 3, (2, 4, 5) => 5, (2, 0, 4) => 1, // near
            (3, 1, 3) => 0, (3, 1, 5) => 2, (3, 5, 7) => 5, (3, 3, 7) => 4, // right
            (4, 2, 3) => 0, (4, 3, 7) => 3, (4, 2, 6) => 1, (4, 6, 7) => 5, // far
            (5, 4, 5) => 2, (5, 5, 7) => 3, (5, 4, 6) => 1, (5, 6, 7) => 4, // top
            _ => panic!(),
        }
    }

    /// Return the vertex opposite from `v0` on the edge of `face` perpendicular to `v0,v1`
    ///
    /// # Panics
    /// This will panic if passed an invalid `(face, edge)` pair.
    fn opposite_vert(face: usize, v0: usize, v1: usize) -> usize {
        match (face, v0, v1) {
            (0, 0, 1) => 2, (0, 1, 0) => 3,     (0, 1, 3) => 0, (0, 3, 1) => 2, // bottom
            (0, 2, 3) => 0, (0, 3, 2) => 1,     (0, 0, 2) => 1, (0, 2, 0) => 3,

            (1, 0, 2) => 4, (1, 2, 0) => 6,     (1, 0, 4) => 2, (1, 4, 0) => 6, // left
            (1, 4, 6) => 0, (1, 6, 4) => 2,     (1, 2, 6) => 0, (1, 6, 2) => 4,

            (2, 0, 1) => 4, (2, 1, 0) => 5,     (2, 1, 5) => 0, (2, 5, 1) => 4, // near
            (2, 4, 5) => 0, (2, 5, 4) => 1,     (2, 0, 4) => 1, (2, 4, 0) => 5,

            (3, 1, 3) => 5, (3, 3, 1) => 7,     (3, 1, 5) => 3, (3, 5, 1) => 7, // right
            (3, 5, 7) => 1, (3, 7, 5) => 3,     (3, 3, 7) => 1, (3, 7, 3) => 5,

            (4, 2, 3) => 6, (4, 3, 2) => 7,     (4, 3, 7) => 2, (4, 7, 3) => 6, // far
            (4, 2, 6) => 3, (4, 6, 2) => 7,     (4, 6, 7) => 2, (4, 7, 6) => 3,

            (5, 4, 5) => 6, (5, 5, 4) => 7,     (5, 4, 6) => 5, (5, 6, 4) => 7, // top
            (5, 6, 7) => 4, (5, 7, 6) => 5,     (5, 5, 7) => 4, (5, 7, 5) => 6,

            _ => unreachable!()
        }
    }

    /// Map a location going off the edge of set space
    ///
    /// This will find the corresponding edge, find the corresponding cell on the connecting face,
    /// and work out the new direction in 2d space that the actor is moving in.
    ///
    /// # Panics
    /// This will panic if the given position does not lie along a face edge, or if the direction
    /// is invalid.
    fn map(&self, pos: (usize, usize), dir: Dir) -> ((usize, usize), Dir) {
        // Find the face - only one face will contain the specified position
        let face = self.faces.iter()
                  .position(|f| f.contains_pos(pos))
                  .expect("Invalid position");

        // Find which edge of the face
        let is_horiz = matches!(dir, Dir::Up | Dir::Down);
        let (v0, v1, c0, _c1) = {
            let (i0, i1) = if is_horiz {
                // find the two verts which match the position Y
                let mut f_idxs = self.faces[face].coords.iter().enumerate()
                                .filter(|(_,c)| c.1 == pos.1)
                                .map(|t| t.0);
                (f_idxs.next().unwrap(), f_idxs.next().unwrap())
            } else {
                // find the two verts which match the position X
                let mut f_idxs = self.faces[face].coords.iter().enumerate()
                                .filter(|(_,c)| c.0 == pos.0)
                                .map(|t| t.0);
                (f_idxs.next().unwrap(), f_idxs.next().unwrap())
            };

            let f = &self.faces[face];
            (f.verts[i0], f.verts[i1], f.coords[i0], f.coords[i1])
        };

        // Find peer face
        let opp_face = Self::opposite_face([v0, v1], face);

        // Find corresponding edge on peer face
        //
        // Note that this is order-sensitive! (pc0, pc1) must be in the same vertex order as
        // (c0, c1) so we can correctly map the position on the source face onto the target edge.
        //
        // This also finds the direction - since the two points are guaranteed to lie on the same
        // axis, the resulting direction is determined by the other two points.
        let (pc0, pc1, dir) = {
            let j0 = self.faces[opp_face].verts.iter().position(|v| *v == v0).unwrap();
            let j1 = self.faces[opp_face].verts.iter().position(|v| *v == v1).unwrap();

            let c_j0 = self.faces[opp_face].coords[j0];
            let c_j1 = self.faces[opp_face].coords[j1];

            // used for finding the direction
            let j2 = (0..4).into_iter().find(|x| *x != j0 && *x != j1).unwrap();
            let c_j2 = self.faces[opp_face].coords[j2];
            let edge_horiz = c_j0.1 == c_j1.1;

            let dir = if edge_horiz { // either up or down, towards j2
                match c_j0.1.cmp(&c_j2.1) {
                    std::cmp::Ordering::Less => Dir::Down,
                    std::cmp::Ordering::Greater => Dir::Up,
                    std::cmp::Ordering::Equal => unreachable!(),
                }
            } else { // either left or right, towards j2
                match c_j0.0.cmp(&c_j2.0) {
                    std::cmp::Ordering::Less => Dir::Right,
                    std::cmp::Ordering::Greater => Dir::Left,
                    std::cmp::Ordering::Equal => unreachable!(),
                }
            };

            (c_j0, c_j1, dir)
        };

        // Find position in the target face: find the distance t that pos is along `(c0, c1)` and
        // interpolate the same distance `t` along `(pc0, pc1)`
        let t = if is_horiz { (pos.0 as isize - c0.0 as isize).abs() }
                else { (pos.1 as isize - c0.1 as isize).abs() };
        assert!(t.is_positive() || t == 0);

        let dest = {
            let dx = pc1.0 as isize - pc0.0 as isize;
            let dy = pc1.1 as isize - pc0.1 as isize;
            (((t*dx.signum()) + pc0.0 as isize) as usize,
             ((t*dy.signum()) + pc0.1 as isize) as usize)
        };

        (dest, dir)
    }
}

fn solve2(input: &Input) -> Result<usize> {
    let mapping = CubeMapping::new(input);
    let mut actor = Actor::new(&input.map);
    for m in &input.directions {
        actor.execute(input, m, |_, pos, dir| {
            let (out_pos, out_dir) = mapping.map(pos.coords(), dir);
            let (unmap_pos, unmap_dir) = mapping.map(out_pos, out_dir.flip());
            assert_eq!(pos.coords(), unmap_pos);
            assert_eq!(unmap_dir, dir.flip());
            (input.map.point(out_pos), out_dir)
        });
    }

    Ok(actor.code())
}

problem!(load_input => Problem => (solve1, solve2));

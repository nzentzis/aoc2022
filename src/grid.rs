#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Grid<T> {
    /// Construct a grid from a flat (row-major) data vector
    ///
    /// The height will be set automatically based on the number of elements and the width.
    ///
    /// # Panics
    /// This will panic if `data.len()` is not divisible by `width`.
    pub fn from_data(data: Vec<T>, width: usize) -> Self {
        assert!(data.len() % width == 0, "Data array is not evenly divisible into a grid");
        Self {
            height: data.len() / width,
            data, width,
        }
    }

    /// Construct a grid by calling a function with each coordinate
    pub fn from_fn<F: Fn(usize, usize) -> T>(width: usize, height: usize, func: F) -> Self {
        let mut data = Vec::with_capacity(width*height);
        for y in 0..height {
            for x in 0..width {
                data.push((func)(x, y));
            }
        }

        Self { data, width, height }
    }

    /// Map the individual cell values through a function, returning a new grid
    pub fn map<U, F: Fn(&T) -> U>(self, func: F) -> Grid<U> {
        let data = self.data.iter().map(func).collect();
        Grid { data, width: self.width, height: self.height }
    }

    /// Get the width of the grid
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the grid
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the value at given coordinates
    ///
    /// # Panics
    /// Panics if the given position is not inside the grid.
    #[inline]
    pub fn get(&self, pos: (usize, usize)) -> &T {
        assert!(pos.0 < self.width && pos.1 < self.height,
                "Attempted to access position ({}, {}) outside grid", pos.0, pos.1);

        &self.data[self.width*pos.1 + pos.0]
    }

    /// Try to get the value at given coordinates
    #[inline]
    pub fn try_get(&self, pos: (usize, usize)) -> Option<&T> {
        if !(pos.0 < self.width && pos.1 < self.height) {
            return None;
        }

        Some(&self.data[self.width*pos.1 + pos.0])
    }

    /// Get mutable reference to the value at given coordinates
    ///
    /// # Panics
    /// Panics if the given position is not inside the grid.
    #[inline]
    pub fn get_mut(&mut self, pos: (usize, usize)) -> &mut T {
        assert!(pos.0 < self.width && pos.1 < self.height,
                "Attempted to access position ({}, {}) outside grid", pos.0, pos.1);

        &mut self.data[self.width*pos.1 + pos.0]
    }

    /// Set the value at given coordinates
    ///
    /// # Panics
    /// Panics if the given position is not inside the grid.
    #[inline]
    pub fn set(&mut self, pos: (usize, usize), val: T) {
        *self.get_mut(pos) = val;
    }

    /// Iterate over all cells in unspecified order
    pub fn cells(&self) -> impl Iterator<Item=&T> {
        self.data.iter()
    }

    /// Consume the grid and return an iterator over all cells in unspecified order
    pub fn into_cells(self) -> impl Iterator<Item=T> {
        self.data.into_iter()
    }

    /// Iterate over each point on the grid in an unspecified order
    pub fn points(&self) -> impl Iterator<Item=GridPoint<T>> {
        (0..self.data.len()).into_iter()
                            .map(|idx| GridPoint {
                                index: idx,
                                coords: (idx % self.width, idx / self.width),
                                grid: self,
                            })
    }

    /// Get a reference to a specific point on the grid
    ///
    /// # Panics
    /// Panics if the given position is not inside the grid
    pub fn point(&self, pos: (usize, usize)) -> GridPoint<T> {
        assert!(pos.0 < self.width && pos.1 < self.height,
                "Attempted to access position ({}, {}) outside grid", pos.0, pos.1);
        GridPoint {
            coords: pos,
            index: pos.0 + pos.1*self.width,
            grid: self,
        }
    }

    /// Get an iterator over the cells in a given row
    ///
    /// # Panics
    /// Panics if the given row is not inside the grid.
    #[inline]
    pub fn row_iter(&self, row: usize) -> impl Iterator<Item=&T> + DoubleEndedIterator {
        assert!(row < self.height, "Attempted to access row outside the grid");

        let start_idx = row*self.width;
        self.data[start_idx..start_idx+self.width].iter()
    }

    /// Get an iterator over mutable references to the cells in a given row
    ///
    /// # Panics
    /// Panics if the given row is not inside the grid.
    #[inline]
    pub fn row_iter_mut(&mut self, row: usize) -> impl Iterator<Item=&mut T> + DoubleEndedIterator {
        assert!(row < self.height, "Attempted to access row outside the grid");

        let start_idx = row*self.width;
        self.data[start_idx..start_idx+self.width].iter_mut()
    }

    /// Get an iterator over the cells in a given column
    ///
    /// # Panics
    /// Panics if the given column is not inside the grid.
    #[inline]
    pub fn col_iter(&self, col: usize) -> impl Iterator<Item=&T> + DoubleEndedIterator {
        assert!(col < self.width, "Attempted to access column outside the grid");

        let start_idx = col;
        let end_idx = self.data.len() - (self.width - col) + 1;
        let content = &self.data[start_idx..end_idx];

        content.iter().step_by(self.width)
    }

    /// Get an iterator over mutable references to the cells in a given column
    ///
    /// # Panics
    /// Panics if the given column is not inside the grid.
    #[inline]
    pub fn col_iter_mut(&mut self, col: usize) -> impl Iterator<Item=&mut T> + DoubleEndedIterator {
        assert!(col < self.width, "Attempted to access column outside the grid");

        let start_idx = col;
        let end_idx = self.data.len() - (self.width - col) + 1;
        let content = &mut self.data[start_idx..end_idx];

        content.iter_mut().step_by(self.width)
    }

    /// Display the grid to the console using a given rendering function
    pub fn show_with<F: Fn(&T) -> char>(&self, func: F) {
        eprintln!();
        for row in self.data.chunks(self.width) {
            for x in row {
                eprint!("{}", (func)(x));
            }
            eprintln!();
        }
    }
}

impl<T: Copy> Grid<T> {
    /// Create a new grid filled with a given value
    pub fn filled(width: usize, height: usize, data: T) -> Self {
        Self { data: vec![data; width*height], width, height }
    }

    /// Create a new grid, the same shape as another one, filled with a given value
    pub fn filled_like<U>(other: &Grid<U>, data: T) -> Self {
        Self::filled(other.width, other.height, data)
    }
}

impl<T: PartialEq<T> + Eq> Grid<T> {
    /// Iterate over grid cells with a given value
    pub fn find(&self, val: T) -> impl Iterator<Item=GridPoint<T>> {
        use std::ops::Deref;

        self.points().filter(move |p| p.deref() == &val)
    }
}

impl std::fmt::Display for Grid<bool> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f)?;
        for row in self.data.chunks(self.width) {
            for b in row {
                write!(f, "{}", if *b { '#' } else { ' ' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// A reference to a specific point on a grid
pub struct GridPoint<'grid, T> {
    index: usize,
    coords: (usize, usize),
    grid: &'grid Grid<T>,
}

impl<T> Clone for GridPoint<'_, T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            coords: self.coords,
            grid: self.grid,
        }
    }
}
impl<T> Copy for GridPoint<'_, T> {}

impl<'g, T> GridPoint<'g, T> {
    /// Get the cell at a given offset relative to this one, if it exists
    pub fn offset(&self, (dx, dy): (isize, isize)) -> Option<Self> {
        let mut index = self.index;
        let (mut x, mut y) = self.coords;

        // bounds check and offset X
        if dx < 0 { // moving left
            let dx = (-dx) as usize;
            x = x.checked_sub(dx)?;
            index -= dx;
        } else { // moving right or not changing X
            let dx = dx as usize;
            x = x.checked_add(dx)?;
            if x >= self.grid.width {
                return None;
            }
            index += dx;
        }

        // bounds check and offset Y
        if dy < 0 { // moving left
            let dy = (-dy) as usize;
            y = y.checked_sub(dy)?;
            index -= self.grid.height * dy;
        } else { // moving right or not changing X
            let dy = dy as usize;
            y = y.checked_add(dy)?;
            if y >= self.grid.height {
                return None;
            }
            index += self.grid.height * dy;
        }

        Some(Self { grid: self.grid, index, coords: (x, y) })
    }

    /// Get the coordinates of this point
    pub fn coords(&self) -> (usize, usize) {
        self.coords
    }

    /// Get the cell to the left of this one, if it exists
    pub fn left(&self) -> Option<Self> {
        if self.coords.0 > 0 {
            Some(Self {
                index: self.index - 1,
                coords: (self.coords.0 - 1, self.coords.1),
                grid: self.grid
            })
        } else {
            None
        }
    }

    /// Get the cell to the right of this one, if it exists
    pub fn right(&self) -> Option<Self> {
        if self.coords.0 < (self.grid.width - 1) {
            Some(Self {
                index: self.index + 1,
                coords: (self.coords.0 + 1, self.coords.1),
                grid: self.grid
            })
        } else {
            None
        }
    }

    /// Walk to the left up to the edge of the grid
    ///
    /// This returns an iterator yielding all points between this one and the left edge of the
    /// grid.
    pub fn walk_left(&self) -> GridPointWalkRow<'g, T> {
        GridPointWalkRow {
            right: false,
            point: *self,
        }
    }

    /// Walk to the right up to the edge of the grid
    ///
    /// This returns an iterator yielding all points between this one and the right edge of the
    /// grid.
    pub fn walk_right(&self) -> GridPointWalkRow<'g, T> {
        GridPointWalkRow {
            right: true,
            point: *self,
        }
    }

    /// Get the cell above this one, if it exists
    pub fn up(&self) -> Option<Self> {
        if self.coords.1 > 0 {
            Some(Self {
                index: self.index - self.grid.width,
                coords: (self.coords.0, self.coords.1 - 1),
                grid: self.grid
            })
        } else {
            None
        }
    }

    /// Get the cell to the right of this one, if it exists
    pub fn down(&self) -> Option<Self> {
        if self.coords.1 < (self.grid.height - 1) {
            Some(Self {
                index: self.index + self.grid.width,
                coords: (self.coords.0, self.coords.1 + 1),
                grid: self.grid
            })
        } else {
            None
        }
    }

    /// Walk upwards to the edge of the grid
    ///
    /// This returns an iterator yielding all points between this one and the top edge of the grid.
    pub fn walk_up(&self) -> GridPointWalkColumn<'g, T> {
        GridPointWalkColumn {
            down: false,
            point: *self,
        }
    }

    /// Walk downwards to the edge of the grid
    ///
    /// This returns an iterator yielding all points between this one and the bottom edge of the
    /// grid.
    pub fn walk_down(&self) -> GridPointWalkColumn<'g, T> {
        GridPointWalkColumn {
            down: true,
            point: *self,
        }
    }
}

impl<'g, T> std::ops::Deref for GridPoint<'g, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.grid.data[self.index]
    }
}

pub struct GridPointWalkRow<'g, T> {
    right: bool,
    point: GridPoint<'g, T>,
}

pub struct GridPointWalkColumn<'g, T> {
    down: bool,
    point: GridPoint<'g, T>,
}

impl<'g, T> Iterator for GridPointWalkRow<'g, T> {
    type Item = GridPoint<'g, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.right {
            self.point = self.point.right()?;
        } else {
            self.point = self.point.left()?;
        }
        Some(self.point)
    }
}

impl<'g, T> Iterator for GridPointWalkColumn<'g, T> {
    type Item = GridPoint<'g, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.down {
            self.point = self.point.down()?;
        } else {
            self.point = self.point.up()?;
        }
        Some(self.point)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn row_iteration() {
        let grid = Grid::from_fn(4, 4, |x, y| x+y);

        assert_eq!(grid.row_iter(0).cloned().collect::<Vec<_>>(), vec![0, 1, 2, 3]);
        assert_eq!(grid.row_iter(1).cloned().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
        assert_eq!(grid.row_iter(2).cloned().collect::<Vec<_>>(), vec![2, 3, 4, 5]);
        assert_eq!(grid.row_iter(3).cloned().collect::<Vec<_>>(), vec![3, 4, 5, 6]);

        assert_eq!(grid.row_iter(0).rev().cloned().collect::<Vec<_>>(), vec![3, 2, 1, 0]);
        assert_eq!(grid.row_iter(1).rev().cloned().collect::<Vec<_>>(), vec![4, 3, 2, 1]);
        assert_eq!(grid.row_iter(2).rev().cloned().collect::<Vec<_>>(), vec![5, 4, 3, 2]);
        assert_eq!(grid.row_iter(3).rev().cloned().collect::<Vec<_>>(), vec![6, 5, 4, 3]);
    }

    #[test]
    fn col_iteration() {
        let grid = Grid::from_fn(4, 4, |x, y| x+y);

        assert_eq!(grid.col_iter(0).cloned().collect::<Vec<_>>(), vec![0, 1, 2, 3]);
        assert_eq!(grid.col_iter(1).cloned().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
        assert_eq!(grid.col_iter(2).cloned().collect::<Vec<_>>(), vec![2, 3, 4, 5]);
        assert_eq!(grid.col_iter(3).cloned().collect::<Vec<_>>(), vec![3, 4, 5, 6]);

        assert_eq!(grid.col_iter(0).rev().cloned().collect::<Vec<_>>(), vec![3, 2, 1, 0]);
        assert_eq!(grid.col_iter(1).rev().cloned().collect::<Vec<_>>(), vec![4, 3, 2, 1]);
        assert_eq!(grid.col_iter(2).rev().cloned().collect::<Vec<_>>(), vec![5, 4, 3, 2]);
        assert_eq!(grid.col_iter(3).rev().cloned().collect::<Vec<_>>(), vec![6, 5, 4, 3]);
    }
}

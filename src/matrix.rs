use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::{GetDisjointMutError, Iter, IterMut},
};

pub struct MatrixArray<const LEN: usize> {
    pub entries: [f32; LEN],
}

impl<const LEN: usize> MatrixArray<LEN> {
    pub fn iter(&self) -> Iter<'_, f32> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, f32> {
        self.entries.iter_mut()
    }
}

impl<const LEN: usize> DerefMut for MatrixArray<LEN> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl<const LEN: usize> Deref for MatrixArray<LEN> {
    type Target = [f32; LEN];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl<const LEN: usize> std::fmt::Debug for MatrixArray<LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.entries.iter()).finish()
    }
}

pub struct Matrix<const NUM_ROWS: usize, const NUM_COLS: usize> {
    pub row_major: [MatrixArray<NUM_COLS>; NUM_ROWS],
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> Matrix<NUM_ROWS, NUM_COLS> {
    pub fn new_row_major(row_major: [[f32; NUM_COLS]; NUM_ROWS]) -> Self {
        Matrix {
            row_major: row_major.map(|row| MatrixArray { entries: row }),
        }
    }

    pub fn col_iter<'a>(&'a self, col_ix: usize) -> MatrixColIter<'a, NUM_ROWS, NUM_COLS> {
        MatrixColIter {
            matrix: self,
            col_ix,
            row_ix: 0,
        }
    }

    pub fn col(&self, col_ix: usize) -> [f32; NUM_ROWS] {
        let mut col = [0.0; NUM_ROWS];
        for (i, entry) in self.col_iter(col_ix).enumerate() {
            col[i] = entry;
        }
        col
    }

    pub fn row(&self, row_ix: usize) -> &MatrixArray<NUM_COLS> {
        &self.row_major[row_ix]
    }

    pub fn row_mut(&mut self, row_ix: usize) -> &mut MatrixArray<NUM_COLS> {
        &mut self.row_major[row_ix]
    }

    /// R_i <-> R_j
    pub fn ero_swap(&mut self, i: usize, j: usize) -> Result<(), GetDisjointMutError> {
        let mut rows = self.row_major.get_disjoint_mut([i, j])?.into_iter();
        let row_i = rows.next().unwrap();
        let row_j = rows.next().unwrap();

        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter_mut()) {
            // i = i + j
            *entry_i += *entry_j;
            // j = i - j
            *entry_j = *entry_i - *entry_j;
            // i = i - j
            *entry_i -= *entry_j;
        }

        Ok(())
    }

    /// R_i = scale * R_i
    pub fn ero_scale(&mut self, i: usize, scale: f32) {
        let row = self.row_mut(i);

        for entry in row.iter_mut() {
            *entry *= scale;
        }
    }

    // R_i = R_i + scale * R_j
    pub fn ero(&mut self, i: usize, scale: f32, j: usize) -> Result<(), GetDisjointMutError> {
        let mut rows = self.row_major.get_disjoint_mut([i, j])?.into_iter();
        let row_i = rows.next().unwrap();
        let row_j = rows.next().unwrap();

        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter()) {
            *entry_i += scale * entry_j;
        }

        Ok(())
    }

    pub const fn num_cols(&self) -> usize {
        NUM_COLS
    }

    pub const fn num_rows(&self) -> usize {
        NUM_ROWS
    }
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> IndexMut<(usize, usize)>
    for Matrix<NUM_ROWS, NUM_COLS>
{
    fn index_mut(&mut self, (row_ix, col_ix): (usize, usize)) -> &mut Self::Output {
        &mut self.row_mut(row_ix)[col_ix]
    }
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> Index<(usize, usize)>
    for Matrix<NUM_ROWS, NUM_COLS>
{
    type Output = f32;

    fn index(&self, (row_ix, col_ix): (usize, usize)) -> &Self::Output {
        &self.row(row_ix)[col_ix]
    }
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> std::fmt::Debug for Matrix<NUM_ROWS, NUM_COLS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.row_major.iter()).finish()
    }
}

pub struct MatrixColIter<'a, const NUM_ROWS: usize, const NUM_COLS: usize> {
    matrix: &'a Matrix<NUM_ROWS, NUM_COLS>,
    col_ix: usize,
    row_ix: usize,
}

impl<'a, const NUM_ROWS: usize, const NUM_COLS: usize> Iterator
    for MatrixColIter<'a, NUM_ROWS, NUM_COLS>
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.row_ix += 1;
        Some(self.matrix.row_major.get(self.row_ix - 1)?[self.col_ix])
    }
}

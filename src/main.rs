use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::{GetDisjointMutError, Iter, IterMut},
};

struct MatrixArray<const LEN: usize> {
    entries: [f32; LEN],
}

impl<const LEN: usize> MatrixArray<LEN> {
    fn zeroed() -> Self {
        MatrixArray {
            entries: [0.0; LEN],
        }
    }

    fn is_leading(&self) -> bool {
        todo!()
    }

    fn iter(&self) -> Iter<'_, f32> {
        self.entries.iter()
    }

    fn iter_mut(&mut self) -> IterMut<'_, f32> {
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

struct Matrix<const NUM_ROWS: usize, const NUM_COLS: usize> {
    row_major: [MatrixArray<NUM_COLS>; NUM_ROWS],
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> Matrix<NUM_ROWS, NUM_COLS> {
    fn new_row_major(row_major: [[f32; NUM_COLS]; NUM_ROWS]) -> Self {
        Matrix {
            row_major: row_major.map(|row| MatrixArray { entries: row }),
        }
    }

    fn col<'a>(&'a self, col_ix: usize) -> MatrixColIter<'a, NUM_ROWS, NUM_COLS> {
        MatrixColIter {
            matrix: self,
            col_ix,
            row_ix: 0,
        }
    }

    fn row(&self, row_ix: usize) -> &MatrixArray<NUM_COLS> {
        &self.row_major[row_ix]
    }

    fn row_mut(&mut self, row_ix: usize) -> &mut MatrixArray<NUM_COLS> {
        &mut self.row_major[row_ix]
    }

    /// R_i <-> R_j
    fn ero_swap(&mut self, i: usize, j: usize) -> Result<(), GetDisjointMutError> {
        let mut rows = self.row_major.get_disjoint_mut([i, j])?.into_iter();
        let row_i = rows.next().unwrap();
        let row_j = rows.next().unwrap();

        // i = i + j
        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter_mut()) {
            *entry_i += *entry_j;
        }

        // j = i - j
        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter_mut()) {
            *entry_j = *entry_i - *entry_j;
        }

        // i = i - j
        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter_mut()) {
            *entry_i -= *entry_j;
        }

        Ok(())
    }

    /// R_i = scale * R_i
    fn ero_scale(&mut self, i: usize, scale: f32) {
        let row = self.row_mut(i);

        for entry in row.iter_mut() {
            *entry *= scale;
        }
    }

    // R_i = R_i + scale * R_j
    fn ero(&mut self, i: usize, scale: f32, j: usize) -> Result<(), GetDisjointMutError> {
        let mut rows = self.row_major.get_disjoint_mut([i, j])?.into_iter();
        let row_i = rows.next().unwrap();
        let row_j = rows.next().unwrap();

        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter()) {
            *entry_i += scale * entry_j;
        }

        Ok(())
    }

    const fn num_cols(&self) -> usize {
        NUM_COLS
    }

    const fn num_rows(&self) -> usize {
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

struct MatrixColIter<'a, const NUM_ROWS: usize, const NUM_COLS: usize> {
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

fn main() {
    println!("Hello, world!");

    let mut matrix = Matrix::new_row_major([
        //
        [1.0, 2.0, 1.0],
        [2.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
    ]);

    let mut current_row = 0;
    let mut current_col = 0;
    let mut leading_cols = Vec::new();

    loop {
        dbg!(&matrix);

        // Find the first column number >= current_row which contains a non-zero entry
        // in rows p..n
        let mut non_zero_row = None;
        for col in current_col..matrix.num_cols() {
            non_zero_row = matrix
                .col(col)
                .skip(current_row)
                .position(|entry| entry != 0.0);

            if non_zero_row.is_some() {
                current_col = col;
                leading_cols.push(current_col);

                break;
            }
        }

        let Some(non_zero_row) = non_zero_row else {
            panic!("stop");
        };

        dbg!(current_row);
        dbg!(current_col);
        dbg!(non_zero_row);

        // e_(non_zero_row, current_col) != 0 by construction
        debug_assert_ne!(matrix[(non_zero_row, current_col)], 0.0);
        // non_zero_row >= current_row
        debug_assert!(non_zero_row >= current_row);

        // Move zero rows down
        // If current_row != non_zero_row, then R_current_row <-> R_i
        if current_row != non_zero_row {
            matrix
                .ero_swap(current_row, non_zero_row)
                .expect("current_row != non_zero_row");
        }

        // e_(current_row, current_col) != 0
        debug_assert_ne!(matrix[(current_row, current_col)], 0.0);

        // R_current_row |-> (1/e_(current_row,current_col)) R_current_row
        // => e_(current_row, current_col) = 1
        matrix.ero_scale(current_row, 1.0 / matrix[(current_row, current_col)]);
        debug_assert_eq!(matrix[(current_row, current_col)], 1.0);

        for i in 0..matrix.num_rows() {
            if i == current_row {
                continue;
            }

            // R_i |-> R_i - e_(i, current_col) * R_current_row
            // => e_(i, current_col) = e_(i, current_col) - e_(i, current_col) * e_(current_row, current_col)
            // => e_(i, current_col) = e_(i, current_col) - e_(i, current_col)
            // => e_(i, current_col) = 0
            matrix
                .ero(i, -matrix[(i, current_col)], current_row)
                .expect("i != current_row");
            debug_assert_eq!(matrix[(i, current_col)], 0.0);
        }

        if current_row == matrix.num_rows() - 1 || current_col == matrix.num_cols() - 1 {
            panic!("stop")
        }

        current_row += 1;
    }
}

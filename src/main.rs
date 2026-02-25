use std::{
    ops::{Deref, DerefMut},
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

    fn iter(&self) -> Iter<f32> {
        self.entries.iter()
    }

    fn iter_mut(&mut self) -> IterMut<f32> {
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

struct Matrix<const NUM_ROWS: usize, const NUM_COLS: usize> {
    row_major: [MatrixArray<NUM_COLS>; NUM_ROWS],
    col_major: [MatrixArray<NUM_ROWS>; NUM_COLS],
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> Matrix<NUM_ROWS, NUM_COLS> {
    fn new_row_major(row_major: [[f32; NUM_COLS]; NUM_ROWS]) -> Self {
        let row_major = row_major.map(|row| MatrixArray { entries: row });
        let mut col_major = std::array::from_fn(|_| MatrixArray::<NUM_ROWS>::zeroed());

        for (row, row_entries) in row_major.iter().enumerate() {
            for (col, entry) in row_entries.iter().enumerate() {
                col_major[col][row] = *entry;
            }
        }

        Matrix {
            row_major,
            col_major,
        }
    }

    fn col(&self, col_idx: usize) -> &MatrixArray<NUM_ROWS> {
        &self.col_major[col_idx]
    }

    fn row(&self, row_idx: usize) -> &MatrixArray<NUM_COLS> {
        &self.row_major[row_idx]
    }

    fn row_mut(&mut self, row_idx: usize) -> &mut MatrixArray<NUM_COLS> {
        &mut self.row_major[row_idx]
    }

    fn swap_rows(&mut self, i: usize, j: usize) -> Result<(), GetDisjointMutError> {
        let Matrix {
            row_major,
            col_major,
        } = self;

        let mut rows = row_major.get_disjoint_mut([i, j])?.into_iter();
        let row_i = rows.next().unwrap();
        let row_j = rows.next().unwrap();

        // i = i + j
        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter_mut()) {
            *entry_i = *entry_i + *entry_j;
        }

        // j = i - j
        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter_mut()) {
            *entry_j = *entry_i - *entry_j;
        }

        // i = i - j
        for (entry_i, entry_j) in Iterator::zip(row_i.iter_mut(), row_j.iter_mut()) {
            *entry_i = *entry_i - *entry_j;
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

fn main() {
    println!("Hello, world!");

    let mut matrix = Matrix::new_row_major([
        //
        [1.0, 2.0, 1.0],
        [2.0, 1.0, 1.0],
    ]);

    let mut current_row = 0;
    let mut current_col = 0;
    let mut leading_cols = Vec::new();
    let mut non_zero_row = None;

    // Find the first column number >= current_row which contains a non-zero entry
    // in rows p..n
    for col in current_col..matrix.num_cols() {
        let q_col = matrix.col(col);

        non_zero_row = q_col[current_row..matrix.num_rows()]
            .iter()
            .position(|&entry| entry != 0.0);

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

    // e_iq != 0
    debug_assert_ne!(matrix.row(non_zero_row)[current_col], 0.0);
    // i >= current_row
    debug_assert!(non_zero_row >= current_row);

    // If current_row != non_zero_row, then R_current_row <-> R_i
    if current_row != non_zero_row {
        matrix
            .swap_rows(current_row, non_zero_row)
            .expect("current_row != non_zero_row");
    }

    // e_pq != 0
    debug_assert_ne!(matrix.row(current_row)[current_col], 0.0);
}

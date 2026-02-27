use crate::matrix::Matrix;

mod matrix;

fn gaussian_eliminate<const NUM_ROWS: usize, const NUM_COLS: usize>(
    matrix: &mut Matrix<NUM_ROWS, NUM_COLS>,
) {
    let mut current_row = 0;
    let mut current_col = 0;
    let mut leading_cols = Vec::new();

    loop {
        dbg!(&matrix);

        // Find the first column number >= current_row which contains a non-zero entry
        // in rows current_row..n
        let mut non_zero_row = None;
        for col in current_col..matrix.num_cols() {
            non_zero_row = matrix
                .col(col)
                .skip(current_row)
                .position(|entry| entry != 0.0)
                .map(|non_zero_row| non_zero_row + current_row);

            if non_zero_row.is_some() {
                current_col = col;
                leading_cols.push(current_col);

                break;
            }
        }

        let Some(non_zero_row) = non_zero_row else {
            return;
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
            // => e_(i, current_col) = e_(i, current_col) - e_(i, current_col) * 1
            // => e_(i, current_col) = 0
            matrix
                .ero(i, -matrix[(i, current_col)], current_row)
                .expect("i != current_row");
            debug_assert_eq!(matrix[(i, current_col)], 0.0);
        }

        if current_row == matrix.num_rows() - 1 || current_col == matrix.num_cols() - 1 {
            return;
        }

        current_row += 1;
    }
}

fn main() {
    println!("Hello, world!");

    let mut matrix = Matrix::new_row_major([
        //
        [1.0, 2.0, 1.0],
        [2.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [6.0, 12.0, 10.0],
    ]);

    gaussian_eliminate(&mut matrix);
}

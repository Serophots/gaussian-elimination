use crate::matrix::Matrix;

mod matrix;

#[derive(Debug)]
pub enum LinearSystemSolution<const NUM_ROWS: usize> {
    Unique([f32; NUM_ROWS]),
    None,
    Many,
}

fn gaussian_eliminate<const NUM_ROWS: usize, const NUM_COLS: usize>(
    matrix: &mut Matrix<NUM_ROWS, NUM_COLS>,
) -> LinearSystemSolution<NUM_ROWS> {
    let mut current_row = 0;
    let mut current_col = 0;
    let mut leading_cols = Vec::new();

    'pivot: loop {
        // Find the first column number >= current_row which contains a non-zero entry
        // in rows current_row..n
        let mut non_zero_row = None;
        for col in current_col..matrix.num_cols() {
            non_zero_row = matrix
                .col_iter(col)
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
            break 'pivot;
        };

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
            break 'pivot;
        }

        current_row += 1;
    }

    if leading_cols
        .last()
        .map(|last| *last == matrix.num_cols() - 1)
        .unwrap_or(false)
    {
        // Last column is leading; the system
        // has an equation 0 x1 + 0 x2 + ... = C where C != 0
        // can never be achieved
        return LinearSystemSolution::None;
    }

    if matrix.num_cols() == leading_cols.len() + 1 {
        // Exactly 1 column of [A|b] is non-leading

        let nonleading = {
            // Quick trick to find the *only* missing number in an array of `n-1` distinct numbers
            // in the range [0,n]
            let n = leading_cols.len() + 1;
            let expected_total = (n * (n + 1)) / 2;
            let actual_total = leading_cols.iter().sum::<usize>();
            (expected_total - actual_total) / 2
        };

        if nonleading == matrix.num_cols() - 1 {
            // The last column b is the only non-leading column
            return LinearSystemSolution::Unique(matrix.col(nonleading));
        }
    }

    return LinearSystemSolution::Many;
}

fn main() {
    let mut matrix = Matrix::new_row_major([
        //
        [2.0, 1.0, -1.0, 8.0],
        [-3.0, -1.0, 2.0, -11.0],
        [-2.0, 1.0, 2.0, -3.0],
    ]);

    let solution = gaussian_eliminate(&mut matrix);
    println!("Solved: {:?}", solution);
}

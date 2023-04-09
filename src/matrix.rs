use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use std::fmt::{self, Display};

pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub matrix: Vec<u8>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Matrix {
            rows,
            cols,
            matrix: vec![0; rows * cols],
        }
    }

    pub fn size(&self) -> usize {
        self.rows * self.cols
    }

    pub fn inverse_idx(&self, idx: usize) -> (usize, usize) {
        let row = idx / self.cols;
        let col = idx % self.cols;
        (row, col)
    }

    pub fn idx(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    pub fn get(&self, row: usize, col: usize) -> u8 {
        let idx = self.idx(row, col);
        self.matrix[idx]
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut u8 {
        let idx = self.idx(row, col);
        &mut self.matrix[idx]
    }

    pub fn set(&mut self, row: usize, col: usize, val: u8) {
        let idx = self.idx(row, col);
        self.matrix[idx] = val;
    }
}

impl Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = if self.get(row, col) == 0 { "." } else { "x" };
                write!(f, "{} ", cell)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Matrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MatrixVisitor;

        impl<'de> Visitor<'de> for MatrixVisitor {
            type Value = Matrix;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 2D matrix represented as a nested list")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Matrix, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut matrix: Vec<Vec<u8>> = Vec::new();

                while let Some(row) = seq.next_element::<Vec<u8>>()? {
                    matrix.push(row);
                }

                let rows = matrix.len();
                let cols = matrix.first().map(|row| row.len()).unwrap_or(0);

                let matrix_data = matrix.into_iter().flatten().collect();

                Ok(Matrix {
                    rows,
                    cols,
                    matrix: matrix_data,
                })
            }
        }

        deserializer.deserialize_seq(MatrixVisitor)
    }
}

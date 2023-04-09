use std::fmt::Display;

pub struct Matrix {
    rows: usize,
    cols: usize,
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

    pub fn set(&mut self, row: usize, col: usize, val: u8) {
        let idx = self.idx(row, col);
        self.matrix[idx] = val;
    }
}

impl Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                write!(f, "{} ", self.get(row, col))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

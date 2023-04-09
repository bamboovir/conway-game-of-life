
use clap::Parser;
use rand::Rng;
use rayon::prelude::*;
use std::{fmt::Display, mem::swap, path::PathBuf};

use crate::matrix::Matrix;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct GameOfLifeArgs {
    #[arg(long, default_value_t = 10)]
    rows: usize,
    #[arg(long, default_value_t = 10)]
    cols: usize,
    #[arg(long, default_value_t = false)]
    loopback: bool,
    #[arg(long)]
    initial_file: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    parallel: bool,
}

pub struct GameOfLife {
    rows: usize,
    cols: usize,
    matrix: Matrix,
    backup_matrix: Matrix,
    ticks: usize,
    parallel: bool,
}

impl GameOfLife {
    pub fn from_args(args: &GameOfLifeArgs) -> Self {
        let mut rng = rand::thread_rng();
        let mut matrix = Matrix::new(args.rows, args.cols);

        for row in 0..args.rows {
            for col in 0..args.cols {
                let val = rng.gen_range(0..=1);
                matrix.set(row, col, val);
            }
        }

        GameOfLife {
            rows: args.rows,
            cols: args.cols,
            matrix,
            backup_matrix: Matrix::new(args.rows, args.cols),
            ticks: 0,
            parallel: args.parallel,
        }
    }

    pub fn tick(&mut self) {
        if self.parallel {
            self.parallel_tick();
        } else {
            self.serial_tick();
        }
    }

    fn serial_tick(&mut self) {
        self.ticks += 1;
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut live_count = 0;

                if row < self.rows - 1
                    && col < self.cols - 1
                    && self.matrix.get(row + 1, col + 1) == 1
                {
                    live_count += 1
                }

                if row > 0 && col > 0 && self.matrix.get(row - 1, col - 1) == 1 {
                    live_count += 1
                }

                if row < self.rows - 1 && col > 0 && self.matrix.get(row + 1, col - 1) == 1 {
                    live_count += 1
                }

                if row > 0 && col < self.cols - 1 && self.matrix.get(row - 1, col + 1) == 1 {
                    live_count += 1
                }

                if col < self.cols - 1 && self.matrix.get(row, col + 1) == 1 {
                    live_count += 1
                }

                if col > 0 && self.matrix.get(row, col - 1) == 1 {
                    live_count += 1
                }

                if row < self.rows - 1 && self.matrix.get(row + 1, col) == 1 {
                    live_count += 1
                }

                if row > 0 && self.matrix.get(row - 1, col) == 1 {
                    live_count += 1
                }

                if live_count < 2 || live_count > 3 {
                    self.backup_matrix.set(row, col, 0);
                } else if (live_count == 2 || live_count == 3) && self.matrix.get(row, col) == 1 {
                    self.backup_matrix.set(row, col, 1);
                } else if live_count == 3 && self.matrix.get(row, col) == 0 {
                    self.backup_matrix.set(row, col, 1);
                }
            }
        }
        swap(&mut self.matrix, &mut self.backup_matrix);
    }

    fn parallel_tick(&mut self) {
        self.ticks += 1;
        let matrix = &self.matrix;

        self.backup_matrix
            .matrix
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, value)| {
                let row = idx / self.cols;
                let col = idx % self.cols;

                let mut live_count = 0;

                if row < self.rows - 1 && col < self.cols - 1 && matrix.get(row + 1, col + 1) == 1 {
                    live_count += 1
                }

                if row > 0 && col > 0 && matrix.get(row - 1, col - 1) == 1 {
                    live_count += 1
                }

                if row < self.rows - 1 && col > 0 && matrix.get(row + 1, col - 1) == 1 {
                    live_count += 1
                }

                if row > 0 && col < self.cols - 1 && matrix.get(row - 1, col + 1) == 1 {
                    live_count += 1
                }

                if col < self.cols - 1 && matrix.get(row, col + 1) == 1 {
                    live_count += 1
                }

                if col > 0 && matrix.get(row, col - 1) == 1 {
                    live_count += 1
                }

                if row < self.rows - 1 && matrix.get(row + 1, col) == 1 {
                    live_count += 1
                }

                if row > 0 && matrix.get(row - 1, col) == 1 {
                    live_count += 1
                }

                if live_count < 2 || live_count > 3 {
                    *value = 0;
                } else if (live_count == 2 || live_count == 3) && self.matrix.get(row, col) == 1 {
                    *value = 1;
                } else if live_count == 3 && self.matrix.get(row, col) == 0 {
                    *value = 1;
                }
            });

        swap(&mut self.matrix, &mut self.backup_matrix);
    }
}

impl Display for GameOfLife {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.matrix)
    }
}
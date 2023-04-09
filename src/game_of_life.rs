use clap::Parser;
use rand::Rng;
use rayon::prelude::*;
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::{fmt::Display, mem::swap, path::PathBuf, slice, thread};

use crate::matrix::Matrix;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct GameOfLifeArgs {
    /// The number of rows of the matrix, invalid if initial_file is specified
    #[arg(long, default_value_t = 10)]
    rows: usize,
    /// The number of columns of the matrix, invalid if initial_file is specified
    #[arg(long, default_value_t = 10)]
    cols: usize,
    /// Whether to loop back at matrix boundaries
    #[arg(long, default_value_t = false)]
    loopback: bool,
    /// 2D array json file of initial matrix state, if not set, a random matrix will be initialized.
    #[arg(long)]
    initial_file: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    /// whether to enable parallelism supported by rayon
    parallel: bool,
    #[arg(long, default_value_t = false)]
    /// Whether to enable parallelism supported by native OS thread
    parallel_naive: bool,
    /// Number of OS threads in parallel_naive strategy
    #[arg(long, default_value_t = 2)]
    workers: usize,
}

pub struct GameOfLife {
    rows: usize,
    cols: usize,
    matrix: Matrix,
    backup_matrix: Matrix,
    ticks: usize,
    parallel: bool,
    parallel_naive: bool,
    workers: usize,
    loopback: bool,
}

impl GameOfLife {
    pub fn from_args(args: &GameOfLifeArgs) -> Self {
        let matrix = match &args.initial_file {
            Some(path) => {
                let file = File::open(path).expect("io exception");
                let reader = BufReader::new(file);
                let matrix: Matrix =
                    serde_json::from_reader(reader).expect("json decode exception");
                matrix
            }
            None => {
                let mut rng = rand::thread_rng();
                let mut matrix = Matrix::new(args.rows, args.cols);

                for row in 0..args.rows {
                    for col in 0..args.cols {
                        let val = rng.gen_range(0..=1);
                        matrix.set(row, col, val);
                    }
                }

                matrix
            }
        };

        let rows = matrix.rows;
        let cols = matrix.cols;

        GameOfLife {
            rows,
            cols,
            matrix,
            backup_matrix: Matrix::new(rows, cols),
            ticks: 0,
            parallel: args.parallel,
            parallel_naive: args.parallel_naive,
            workers: args.workers,
            loopback: args.loopback,
        }
    }

    pub fn tick(&mut self) {
        if self.parallel_naive {
            self.parallel_naive_tick();
        } else if self.parallel {
            self.parallel_tick();
        } else {
            self.serial_tick();
        }
    }

    fn serial_tick(&mut self) {
        self.ticks += 1;

        for idx in 0..self.matrix.size() {
            let value = self.backup_matrix.matrix.get_mut(idx).unwrap();
            Self::write_next_tick_state(self.loopback, &self.matrix, idx, value);
        }

        swap(&mut self.matrix, &mut self.backup_matrix);
    }

    fn parallel_tick(&mut self) {
        self.ticks += 1;

        let matrix = &self.matrix;
        let loopback = self.loopback;

        self.backup_matrix
            .matrix
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, value)| {
                Self::write_next_tick_state(loopback, &matrix, idx, value);
            });

        swap(&mut self.matrix, &mut self.backup_matrix);
    }

    fn parallel_naive_tick(&mut self) {
        self.ticks += 1;

        // pass reference to a stack-allocated variable to threads
        thread::scope(|scope| {
            let mut workers = vec![];
            let chunk_size = self.matrix.size() / self.workers;
            let matrix_arc = Arc::new(&self.matrix);

            for i in 0..self.workers {
                let start = i * chunk_size;
                let end = if i == self.workers - 1 {
                    self.matrix.size()
                } else {
                    start + chunk_size
                };
                let loopback = self.loopback;
                let backup_matrix_ptr_wrapper =
                    ThreadPtrWrapper(self.backup_matrix.matrix.as_mut_ptr());
                let matrix = matrix_arc.clone();

                let worker = scope.spawn(move || unsafe {
                    // inside of the closure to avoid the smarter new fine-grained closure capturing.
                    let _ = &backup_matrix_ptr_wrapper;
                    let backup_matrix_ptr = backup_matrix_ptr_wrapper.0;
                    let slice =
                        slice::from_raw_parts_mut(backup_matrix_ptr.add(start), end - start);

                    for idx in start..end {
                        let value = &mut slice[idx - start];
                        Self::write_next_tick_state(loopback, &matrix, idx, value);
                    }
                });

                workers.push(worker)
            }

            for worker in workers {
                worker.join().unwrap();
            }
        });

        swap(&mut self.matrix, &mut self.backup_matrix);
    }

    fn write_next_tick_state(loopback: bool, matrix: &Matrix, idx: usize, value: &mut u8) {
        if loopback {
            Self::write_loopback_next_tick_state(matrix, idx, value);
        } else {
            Self::write_terminate_next_tick_state(matrix, idx, value);
        }
    }

    fn write_terminate_next_tick_state(matrix: &Matrix, idx: usize, value: &mut u8) {
        let rows = matrix.rows;
        let cols = matrix.cols;
        let (row, col) = matrix.inverse_idx(idx);

        let mut live_count = 0;

        if row < rows - 1 && col < cols - 1 && matrix.get(row + 1, col + 1) == 1 {
            live_count += 1
        }

        if row > 0 && col > 0 && matrix.get(row - 1, col - 1) == 1 {
            live_count += 1
        }

        if row < rows - 1 && col > 0 && matrix.get(row + 1, col - 1) == 1 {
            live_count += 1
        }

        if row > 0 && col < cols - 1 && matrix.get(row - 1, col + 1) == 1 {
            live_count += 1
        }

        if col < cols - 1 && matrix.get(row, col + 1) == 1 {
            live_count += 1
        }

        if col > 0 && matrix.get(row, col - 1) == 1 {
            live_count += 1
        }

        if row < rows - 1 && matrix.get(row + 1, col) == 1 {
            live_count += 1
        }

        if row > 0 && matrix.get(row - 1, col) == 1 {
            live_count += 1
        }

        *value = if matrix.get(row, col) == 1 {
            if live_count < 2 || live_count > 3 {
                0
            } else {
                1
            }
        } else {
            if live_count == 3 {
                1
            } else {
                0
            }
        }
    }

    fn write_loopback_next_tick_state(matrix: &Matrix, idx: usize, value: &mut u8) {
        let rows = matrix.rows;
        let cols = matrix.cols;
        let (row, col) = matrix.inverse_idx(idx);

        let mut live_count = 0;

        // loopback
        let row_inc = if row == rows - 1 { 0 } else { row + 1 };
        let row_dec = if row == 0 { rows - 1 } else { row - 1 };
        let col_inc = if col == cols - 1 { 0 } else { col + 1 };
        let col_dec = if col == 0 { cols - 1 } else { col - 1 };

        if matrix.get(row_inc, col_inc) == 1 {
            live_count += 1
        }

        if matrix.get(row_dec, col_dec) == 1 {
            live_count += 1
        }

        if matrix.get(row_inc, col_dec) == 1 {
            live_count += 1
        }

        if matrix.get(row_dec, col_inc) == 1 {
            live_count += 1
        }

        if matrix.get(row, col_inc) == 1 {
            live_count += 1
        }

        if matrix.get(row, col_dec) == 1 {
            live_count += 1
        }

        if matrix.get(row_inc, col) == 1 {
            live_count += 1
        }

        if matrix.get(row_dec, col) == 1 {
            live_count += 1
        }

        *value = if matrix.get(row, col) == 1 {
            if live_count < 2 || live_count > 3 {
                0
            } else {
                1
            }
        } else {
            if live_count == 3 {
                1
            } else {
                0
            }
        }
    }
}

impl Display for GameOfLife {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "size: {} x {} \nticks: {} \n{}",
            self.rows, self.cols, self.ticks, self.matrix
        )
    }
}

// sharing raw pointer wrappers among threads
struct ThreadPtrWrapper(*mut u8);
unsafe impl Sync for ThreadPtrWrapper {}
unsafe impl Send for ThreadPtrWrapper {}

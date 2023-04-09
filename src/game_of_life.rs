use clap::Parser;
use rand::Rng;
use rayon::prelude::*;
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::{cmp::min, fmt::Display, mem::swap, path::PathBuf, slice, thread};

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
    #[arg(long, default_value_t = false)]
    parallel_naive: bool,
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
            Self::write_next_tick_state(&self.matrix, idx, value);
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
                Self::write_next_tick_state(&matrix, idx, value);
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
                let end = min(start + chunk_size, self.matrix.size());
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
                        Self::write_next_tick_state(&matrix, idx, value);
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

    fn write_next_tick_state(matrix: &Matrix, idx: usize, value: &mut u8) {
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

        if live_count < 2 || live_count > 3 {
            *value = 0;
        } else if (live_count == 2 || live_count == 3) && matrix.get(row, col) == 1 {
            *value = 1;
        } else if live_count == 3 && matrix.get(row, col) == 0 {
            *value = 1;
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

use ansi_escapes;
use clap::Parser;
use conway_game_of_life::game_of_life::{GameOfLife, GameOfLifeArgs};
use std::{thread, time::Duration};

fn main() {
    let args = GameOfLifeArgs::parse();
    let mut game = GameOfLife::from_args(&args);

    loop {
        print!("{}", ansi_escapes::ClearScreen);
        println!("{game}");
        game.tick();
        thread::sleep(Duration::from_secs(1));
    }
}

use ansi_escapes;
use clap::Parser;
use conway_game_of_life::game_of_life::{GameOfLife, GameOfLifeArgs};
use std::{thread, time::Duration};

fn main() {
    let args = GameOfLifeArgs::parse();
    let mut game = GameOfLife::from_args(&args);

    println!("args: {:#?}", args);
    println!("{game}");

    loop {
        thread::sleep(Duration::from_secs(1));
        print!("{}", ansi_escapes::ClearScreen);
        game.tick();
        println!("{game}");
    }
}

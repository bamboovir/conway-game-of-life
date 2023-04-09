# Conway's Game of Life

This repository contains an implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) in Rust.

```bash
Usage: conway-game-of-life [OPTIONS]

Options:
      --rows <ROWS>                  [default: 10]
      --cols <COLS>                  [default: 10]
      --loopback                     
      --initial-file <INITIAL_FILE>  
      --parallel                     
      --parallel-naive               
      --workers <WORKERS>            [default: 2]
  -h, --help                         Print help
  -V, --version                      Print version
```

```bash
conway-game-of-life \
  --parallel \
  --initial-file assets/oscillators/blinker.json
```

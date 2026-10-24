# Tetris

A complete terminal Tetris game built with Rust and Ratatui.

## Features

- Marathon, Sprint, and Ultra game modes
- Per-mode personal bests saved between sessions
- Super Rotation System with wall kicks
- Seven-bag randomizer
- Hold piece and five-piece preview queue
- Ghost piece and lock delay with move resets
- Line clear flashes and hard drop trails
- T-spin detection
- Combos, back-to-back clears, and perfect clears
- Guideline gravity and level progression
- Pause, restart, and result screens

## Requirements

- A recent stable Rust toolchain
- A terminal with Unicode support
- At least 42 columns and 24 rows

## Getting Started

```sh
git clone https://github.com/uint82/tetris.git
cd tetris
cargo run --release
```

To build a standalone binary:

```sh
cargo build --release
./target/release/tetris
```

## Game Modes

- Marathon: endless play until you top out
- Sprint: clear 40 lines as fast as possible
- Ultra: score as much as possible in two minutes

## Personal Bests

Marathon and Ultra track the highest score. Sprint tracks the fastest completed run. Records are saved automatically in the platform data directory.

## Controls

### Mode Selection

| Key | Action |
| --- | --- |
| Up arrow or K | Select previous mode |
| Down arrow or J | Select next mode |
| Enter or Space | Start selected mode |
| 1, 2, or 3 | Start Marathon, Sprint, or Ultra |
| Q or Ctrl+C | Quit |

### In Game

| Key | Action |
| --- | --- |
| Left arrow or A | Move left |
| Right arrow or D | Move right |
| Down arrow, S, or J | Soft drop |
| Space | Hard drop |
| Up arrow, X, or L | Rotate clockwise |
| Z or H | Rotate counterclockwise |
| C | Hold piece |
| P or Esc | Pause or resume |
| R or Enter | Restart |
| M | Return to menu from pause or results |
| Q or Ctrl+C | Quit |

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## License

This project is open source under the MIT License. See [LICENSE](LICENSE).

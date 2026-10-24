# Tetris

A complete terminal Tetris game built with Rust and Ratatui.

## Features

- Marathon, Sprint, Ultra, and a daily challenge
- Per-mode personal bests saved between sessions
- Fair 3-2-1 countdown before timed runs
- Date-seeded daily challenge with a daily best
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
- Daily challenge: a seeded two-minute score run using the current UTC date

Marathon starts immediately. Sprint, Ultra, and the daily challenge start after a 3-2-1 countdown, with their clocks starting at zero.

## Personal Bests

Marathon and Ultra track the highest score. Sprint tracks the fastest completed run. The daily challenge tracks the highest score for the current UTC date. Records are saved automatically in the platform data directory.

## Controls

### Mode Selection

| Key | Action |
| --- | --- |
| Up arrow or K | Select previous option |
| Down arrow or J | Select next option |
| Enter or Space | Start selected option |
| 1, 2, 3, or 4 | Start Marathon, Sprint, Ultra, or the daily challenge |
| D | Start the daily challenge |
| Q or Ctrl+C | Quit |

### In Game

| Key | Action |
| --- | --- |
| Left arrow, A, or H | Move left |
| Right arrow, D, or L | Move right |
| Down arrow or S | Soft drop |
| Space | Hard drop |
| Up arrow, X, or K | Rotate clockwise |
| Z or J | Rotate counterclockwise |
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

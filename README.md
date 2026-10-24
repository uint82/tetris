# Tetris

A complete terminal Tetris game built with Rust and Ratatui.

## Features

- Marathon, Sprint, Time, Classic, and a daily challenge
- Per-mode personal bests saved between sessions
- Fair 3-2-1 countdown before timed runs
- NES-style Classic rules with classic scoring
- Date-seeded daily challenge with a daily best
- Super Rotation System with wall kicks for modern modes
- Seven-bag randomizer for modern modes
- Hold, ghost, and lock delay for modern modes
- Five-piece preview queue
- Line clear flashes and hard drop trails
- T-spin, combo, back-to-back, and perfect clear scoring for modern modes
- Guideline and NES-style gravity
- Pause, restart, and result screens

## Requirements

- A recent stable Rust toolchain
- A terminal with Unicode support
- At least 42 columns and 26 rows

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
- Time: score as much as possible in two minutes
- Classic: endless NES-style play with no hold, ghost, hard drop, wall kicks, or lock delay
- Daily challenge: a seeded two-minute Time run using the current UTC date

Marathon and Classic start immediately. Sprint, Time, and the daily challenge start after a 3-2-1 countdown, with their clocks starting at zero.

## Personal Bests

Marathon, Time, and Classic track the highest score. Sprint tracks the fastest completed run. The daily challenge tracks the highest score for the current UTC date. Records are saved automatically in the platform data directory.

## Controls

### Mode Selection

| Key | Action |
| --- | --- |
| Up arrow or K | Select previous option |
| Down arrow or J | Select next option |
| Enter or Space | Start selected option |
| 1, 2, 3, 4, or 5 | Start Marathon, Sprint, Time, Classic, or the daily challenge |
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

Classic uses NES-style scoring, a classic randomizer, and level-based gravity. Hold, ghost, hard drop, wall kicks, lock delay, T-spin scoring, and combo scoring are disabled.

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## License

This project is open source under the MIT License. See [LICENSE](LICENSE).

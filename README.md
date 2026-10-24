# Tetris

A complete terminal Tetris game built with Rust and Ratatui.

## Features

- Super Rotation System with wall kicks
- Seven-bag randomizer
- Hold piece and five-piece preview queue
- Ghost piece and lock delay with move resets
- T-spin detection
- Combos, back-to-back clears, and perfect clears
- Guideline gravity and level progression
- Pause, restart, and game over screens

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

## Controls

| Key | Action |
| --- | --- |
| Left arrow or A | Move left |
| Right arrow or D | Move right |
| Down arrow or S | Soft drop |
| Space | Hard drop |
| Up arrow or X | Rotate clockwise |
| Z | Rotate counterclockwise |
| C | Hold piece |
| P or Esc | Pause or resume |
| R or Enter | Restart |
| Q or Ctrl+C | Quit |

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## License

This project is open source under the MIT License. See [LICENSE](LICENSE).

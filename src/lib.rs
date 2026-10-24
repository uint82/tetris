mod app;
mod bag;
mod board;
mod effects;
mod game;
mod input;
mod kicks;
mod mode;
mod piece;
mod records;
mod scoring;
mod theme;
mod tspin;
mod ui;

pub fn run() -> std::io::Result<()> {
    app::run()
}

use ratatui::style::{Color, Modifier, Style};

use crate::piece::PieceKind;

pub fn piece_color(kind: PieceKind) -> Color {
    match kind {
        PieceKind::I => Color::Rgb(0, 230, 230),
        PieceKind::O => Color::Rgb(240, 220, 0),
        PieceKind::T => Color::Rgb(170, 0, 240),
        PieceKind::S => Color::Rgb(0, 220, 80),
        PieceKind::Z => Color::Rgb(235, 50, 50),
        PieceKind::J => Color::Rgb(60, 100, 240),
        PieceKind::L => Color::Rgb(245, 150, 30),
    }
}

pub fn border() -> Style {
    Style::new().fg(Color::Rgb(85, 90, 110))
}

pub fn title() -> Style {
    Style::new()
        .fg(Color::Rgb(200, 220, 255))
        .add_modifier(Modifier::BOLD)
}

pub fn label() -> Style {
    Style::new().fg(Color::Gray)
}

pub fn value() -> Style {
    Style::new().fg(Color::White).add_modifier(Modifier::BOLD)
}

pub fn ghost() -> Style {
    Style::new().fg(Color::Rgb(75, 80, 100))
}

pub fn line_flash(bright: bool) -> Style {
    let color = if bright {
        Color::White
    } else {
        Color::Rgb(255, 190, 60)
    };
    Style::new().fg(color).add_modifier(Modifier::BOLD)
}

pub fn help() -> Style {
    Style::new().fg(Color::DarkGray)
}

pub fn best() -> Style {
    Style::new().fg(Color::Green)
}

pub fn pause_color() -> Color {
    Color::Yellow
}

pub fn countdown_color() -> Color {
    Color::Yellow
}

pub fn game_over_color() -> Color {
    Color::Red
}

pub fn finished_color() -> Color {
    Color::Green
}

pub fn accent() -> Style {
    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
}

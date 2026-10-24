use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::common::{format_best, render_too_small};
use crate::mode::{Mode, ModeMenu};
use crate::records::PersonalBests;
use crate::theme;

const MENU_WIDTH: u16 = 42;
const MENU_HEIGHT: u16 = 18;
const OPTION_WIDTH: usize = 37;
const DESCRIPTION_INDENT: usize = 4;

pub fn draw(frame: &mut Frame, menu: &ModeMenu, bests: &PersonalBests) {
    let area = frame.area();
    if area.width < MENU_WIDTH || area.height < MENU_HEIGHT {
        render_too_small(frame, area, MENU_WIDTH, MENU_HEIGHT);
        return;
    }
    let lines = menu_lines(menu, bests);
    let height = lines.len() as u16;
    let content = area.centered(Constraint::Length(MENU_WIDTH), Constraint::Length(height));
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), content);
}

fn menu_lines(menu: &ModeMenu, bests: &PersonalBests) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::default(),
        Line::from("TETRIS").style(theme::title()),
        Line::from("Choose a game mode").style(theme::help()),
        Line::default(),
    ];
    for mode in Mode::ALL {
        lines.push(mode_line(mode, menu.selected() == mode));
        lines.push(description_line(mode));
        lines.push(best_line(mode, bests));
        lines.push(Line::default());
    }
    lines.extend(footer_lines());
    lines
}

fn mode_line(mode: Mode, selected: bool) -> Line<'static> {
    let marker = if selected { "> " } else { "  " };
    let text = format!(
        "{marker}{:<width$}",
        mode.label(),
        width = OPTION_WIDTH - marker.len()
    );
    if selected {
        Line::from(text).style(theme::accent())
    } else {
        Line::from(text).style(theme::value())
    }
}

fn description_line(mode: Mode) -> Line<'static> {
    let text = format!(
        "{:indent$}{:<width$}",
        "",
        mode.description(),
        indent = DESCRIPTION_INDENT,
        width = OPTION_WIDTH - DESCRIPTION_INDENT
    );
    Line::from(text).style(theme::help())
}

fn best_line(mode: Mode, bests: &PersonalBests) -> Line<'static> {
    let best = bests.best(mode);
    let text = match best {
        Some(record) => format!("    Best {}", format_best(mode, record)),
        None => "    Best --".to_string(),
    };
    let text = format!("{text:<width$}", width = OPTION_WIDTH);
    if best.is_some() {
        Line::from(text).style(theme::best())
    } else {
        Line::from(text).style(theme::help())
    }
}

fn footer_lines() -> Vec<Line<'static>> {
    vec![
        Line::from("Up/Down select   Enter start   Q quit").style(theme::help()),
        Line::from("1-3 quick start").style(theme::help()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn draws_the_mode_menu() {
        let backend = TestBackend::new(MENU_WIDTH, MENU_HEIGHT);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let menu = ModeMenu::new();
        let bests = PersonalBests::default();
        terminal
            .draw(|frame| draw(frame, &menu, &bests))
            .expect("menu renders");
    }

    #[test]
    fn draws_a_small_terminal_without_panicking() {
        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let menu = ModeMenu::new();
        let bests = PersonalBests::default();
        terminal
            .draw(|frame| draw(frame, &menu, &bests))
            .expect("menu renders");
    }
}

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Clear, Paragraph};

use crate::theme;

pub fn render_too_small(frame: &mut Frame, area: Rect, required_width: u16, required_height: u16) {
    frame.render_widget(Clear, area);
    let width = required_width.min(area.width);
    let popup = area.centered(Constraint::Length(width), Constraint::Length(2));
    let lines = vec![
        Line::from("Terminal too small").style(theme::value()),
        Line::from(format!(
            "Required size: {required_width} x {required_height}"
        ))
        .style(theme::help()),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), popup);
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn formats_minutes_and_seconds() {
        assert_eq!(format_duration(Duration::ZERO), "0:00");
        assert_eq!(format_duration(Duration::from_secs(65)), "1:05");
        assert_eq!(format_duration(Duration::from_secs(600)), "10:00");
    }
}

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Clear, Paragraph};

use crate::mode::Mode;
use crate::records::Record;
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

pub fn format_best(mode: Mode, record: Record) -> String {
    match mode {
        Mode::Sprint => format_duration(record.elapsed()),
        Mode::Marathon | Mode::Ultra => record.score().to_string(),
    }
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

    #[test]
    fn formats_bests_by_mode() {
        let record = Record::new(1200, 40, 4, Duration::from_secs(75));
        assert_eq!(format_best(Mode::Marathon, record), "1200");
        assert_eq!(format_best(Mode::Sprint, record), "1:15");
        assert_eq!(format_best(Mode::Ultra, record), "1200");
    }
}

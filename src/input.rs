use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::Action;

pub fn action(key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char(character) if character.eq_ignore_ascii_case(&'c') => Some(Action::Quit),
            _ => None,
        };
    }
    match key.code {
        KeyCode::Left => Some(Action::MoveLeft),
        KeyCode::Right => Some(Action::MoveRight),
        KeyCode::Down => Some(Action::SoftDrop),
        KeyCode::Up => Some(Action::RotateClockwise),
        KeyCode::Char(' ') => Some(Action::HardDrop),
        KeyCode::Enter => Some(Action::Restart),
        KeyCode::Esc => Some(Action::TogglePause),
        KeyCode::Char(character) => letter_action(character),
        _ => None,
    }
}

fn letter_action(character: char) -> Option<Action> {
    match character.to_ascii_lowercase() {
        'a' => Some(Action::MoveLeft),
        'd' => Some(Action::MoveRight),
        's' => Some(Action::SoftDrop),
        'x' => Some(Action::RotateClockwise),
        'z' => Some(Action::RotateCounterClockwise),
        'c' => Some(Action::Hold),
        'p' => Some(Action::TogglePause),
        'r' => Some(Action::Restart),
        'q' => Some(Action::Quit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn arrows_map_to_movement() {
        assert_eq!(action(key(KeyCode::Left)), Some(Action::MoveLeft));
        assert_eq!(action(key(KeyCode::Right)), Some(Action::MoveRight));
        assert_eq!(action(key(KeyCode::Down)), Some(Action::SoftDrop));
        assert_eq!(action(key(KeyCode::Up)), Some(Action::RotateClockwise));
    }

    #[test]
    fn letters_map_to_game_actions() {
        assert_eq!(
            action(key(KeyCode::Char('z'))),
            Some(Action::RotateCounterClockwise)
        );
        assert_eq!(action(key(KeyCode::Char('c'))), Some(Action::Hold));
        assert_eq!(action(key(KeyCode::Char('P'))), Some(Action::TogglePause));
    }

    #[test]
    fn control_c_quits() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action(event), Some(Action::Quit));
    }

    #[test]
    fn unknown_keys_are_ignored() {
        assert_eq!(action(key(KeyCode::Char('8'))), None);
    }
}

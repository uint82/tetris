use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::Action;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Previous,
    Next,
    Start,
    StartMode(usize),
    Quit,
}

pub fn action(key: KeyEvent) -> Option<Action> {
    if quit_shortcut(key) {
        return Some(Action::Quit);
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return None;
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

pub fn menu_action(key: KeyEvent) -> Option<MenuAction> {
    if quit_shortcut(key) {
        return Some(MenuAction::Quit);
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return None;
    }
    match key.code {
        KeyCode::Up => Some(MenuAction::Previous),
        KeyCode::Down => Some(MenuAction::Next),
        KeyCode::Enter | KeyCode::Char(' ') => Some(MenuAction::Start),
        KeyCode::Char(character) => menu_letter_action(character),
        _ => None,
    }
}

fn letter_action(character: char) -> Option<Action> {
    match character.to_ascii_lowercase() {
        'a' => Some(Action::MoveLeft),
        'd' => Some(Action::MoveRight),
        's' | 'j' => Some(Action::SoftDrop),
        'x' | 'l' => Some(Action::RotateClockwise),
        'z' | 'h' => Some(Action::RotateCounterClockwise),
        'c' => Some(Action::Hold),
        'p' => Some(Action::TogglePause),
        'r' => Some(Action::Restart),
        'm' => Some(Action::Menu),
        'q' => Some(Action::Quit),
        _ => None,
    }
}

fn menu_letter_action(character: char) -> Option<MenuAction> {
    match character.to_ascii_lowercase() {
        'k' => Some(MenuAction::Previous),
        'j' => Some(MenuAction::Next),
        '1' => Some(MenuAction::StartMode(0)),
        '2' => Some(MenuAction::StartMode(1)),
        '3' => Some(MenuAction::StartMode(2)),
        'q' => Some(MenuAction::Quit),
        _ => None,
    }
}

fn quit_shortcut(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(
            key.code,
            KeyCode::Char(character) if character.eq_ignore_ascii_case(&'c')
        )
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
    fn vim_keys_map_to_rotation_and_soft_drop() {
        assert_eq!(
            action(key(KeyCode::Char('h'))),
            Some(Action::RotateCounterClockwise)
        );
        assert_eq!(
            action(key(KeyCode::Char('l'))),
            Some(Action::RotateClockwise)
        );
        assert_eq!(action(key(KeyCode::Char('j'))), Some(Action::SoftDrop));
        assert_eq!(action(key(KeyCode::Char('k'))), None);
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

    #[test]
    fn m_returns_to_the_menu() {
        assert_eq!(action(key(KeyCode::Char('m'))), Some(Action::Menu));
    }

    #[test]
    fn menu_arrows_and_letters_navigate() {
        assert_eq!(menu_action(key(KeyCode::Up)), Some(MenuAction::Previous));
        assert_eq!(
            menu_action(key(KeyCode::Char('k'))),
            Some(MenuAction::Previous)
        );
        assert_eq!(menu_action(key(KeyCode::Down)), Some(MenuAction::Next));
        assert_eq!(menu_action(key(KeyCode::Char('j'))), Some(MenuAction::Next));
    }

    #[test]
    fn menu_start_keys_select_a_mode() {
        assert_eq!(menu_action(key(KeyCode::Enter)), Some(MenuAction::Start));
        assert_eq!(
            menu_action(key(KeyCode::Char(' '))),
            Some(MenuAction::Start)
        );
        assert_eq!(
            menu_action(key(KeyCode::Char('1'))),
            Some(MenuAction::StartMode(0))
        );
        assert_eq!(
            menu_action(key(KeyCode::Char('2'))),
            Some(MenuAction::StartMode(1))
        );
        assert_eq!(
            menu_action(key(KeyCode::Char('3'))),
            Some(MenuAction::StartMode(2))
        );
    }

    #[test]
    fn menu_quits_with_q_or_control_c() {
        assert_eq!(menu_action(key(KeyCode::Char('q'))), Some(MenuAction::Quit));
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(menu_action(event), Some(MenuAction::Quit));
    }
}

use crate::mode::Mode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuChoice {
    Mode(Mode),
    Daily,
}

impl MenuChoice {
    pub const ALL: [Self; 4] = [
        Self::Mode(Mode::Marathon),
        Self::Mode(Mode::Sprint),
        Self::Mode(Mode::Ultra),
        Self::Daily,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Mode(mode) => mode.label(),
            Self::Daily => "DAILY CHALLENGE",
        }
    }

    pub const fn record_mode(self) -> Mode {
        match self {
            Self::Mode(mode) => mode,
            Self::Daily => Mode::Ultra,
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Self::Mode(Mode::Marathon) => 0,
            Self::Mode(Mode::Sprint) => 1,
            Self::Mode(Mode::Ultra) => 2,
            Self::Daily => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModeMenu {
    selected: usize,
}

impl ModeMenu {
    pub const fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn at(choice: MenuChoice) -> Self {
        Self {
            selected: choice.index(),
        }
    }

    pub fn selected(&self) -> MenuChoice {
        MenuChoice::ALL[self.selected]
    }

    pub fn previous(&mut self) {
        self.selected = self
            .selected
            .checked_sub(1)
            .unwrap_or(MenuChoice::ALL.len() - 1);
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % MenuChoice::ALL.len();
    }
}

impl Default for ModeMenu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_starts_at_marathon() {
        assert_eq!(ModeMenu::new().selected(), MenuChoice::Mode(Mode::Marathon));
    }

    #[test]
    fn menu_navigation_wraps() {
        let mut menu = ModeMenu::new();
        menu.previous();
        assert_eq!(menu.selected(), MenuChoice::Daily);
        menu.next();
        assert_eq!(menu.selected(), MenuChoice::Mode(Mode::Marathon));
    }

    #[test]
    fn menu_can_focus_any_choice() {
        for choice in MenuChoice::ALL {
            assert_eq!(ModeMenu::at(choice).selected(), choice);
        }
    }

    #[test]
    fn choices_have_distinct_labels() {
        for (index, choice) in MenuChoice::ALL.iter().enumerate() {
            for other in &MenuChoice::ALL[index + 1..] {
                assert_ne!(choice.label(), other.label());
            }
        }
    }
}

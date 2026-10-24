use std::io;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};

use crate::game::{Action, Game, Status};
use crate::input::{self, MenuAction};
use crate::mode::{Mode, ModeMenu};
use crate::records::{Record, Records};
use crate::ui;

const FRAME_TIME: Duration = Duration::from_millis(16);
const MAX_FRAME_TIME: Duration = Duration::from_millis(100);

pub fn run() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

enum Screen {
    Menu(ModeMenu),
    Game(Box<Game>),
}

struct App {
    screen: Screen,
    records: Records,
    result_recorded: bool,
    new_best: bool,
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self::with_records(Records::load())
    }

    fn with_records(records: Records) -> Self {
        Self {
            screen: Screen::Menu(ModeMenu::new()),
            records,
            result_recorded: false,
            new_best: false,
            quit: false,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| self.draw(frame))?;
        while !self.quit {
            let frame_start = Instant::now();
            self.handle_events(FRAME_TIME)?;
            self.update(frame_start.elapsed().min(MAX_FRAME_TIME));
            if self.quit {
                break;
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn update(&mut self, dt: Duration) {
        if let Some(game) = self.game_mut() {
            game.update(dt);
        }
        self.record_result_if_needed();
    }

    fn draw(&self, frame: &mut Frame) {
        match &self.screen {
            Screen::Menu(menu) => ui::draw_menu(frame, menu, self.records.bests()),
            Screen::Game(game) => {
                ui::draw_game(frame, game.as_ref(), self.records.bests(), self.new_best)
            }
        }
    }

    fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if !event::poll(timeout)? {
            return Ok(());
        }
        loop {
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
            if !event::poll(Duration::ZERO)? {
                return Ok(());
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        if matches!(&self.screen, Screen::Menu(_)) {
            self.handle_menu_key(key);
        } else {
            self.handle_game_key(key);
        }
    }

    fn handle_menu_key(&mut self, key: KeyEvent) {
        let Some(action) = input::menu_action(key) else {
            return;
        };
        match action {
            MenuAction::Previous => self.apply_to_menu(ModeMenu::previous),
            MenuAction::Next => self.apply_to_menu(ModeMenu::next),
            MenuAction::Start => self.start_selected_mode(),
            MenuAction::StartMode(index) => self.start_mode_at(index),
            MenuAction::Quit => self.quit = true,
        }
    }

    fn handle_game_key(&mut self, key: KeyEvent) {
        let Some(action) = input::action(key) else {
            return;
        };
        self.record_result_if_needed();
        match action {
            Action::Quit => self.quit = true,
            Action::Menu => self.return_to_menu(),
            Action::Restart => self.restart_game(),
            Action::TogglePause => self.apply_to_game(Game::toggle_pause),
            action => self.apply_to_game(|game| game.apply(action)),
        }
        self.record_result_if_needed();
    }

    fn start_selected_mode(&mut self) {
        if let Some(mode) = self.menu().map(ModeMenu::selected) {
            self.start_game(mode);
        }
    }

    fn start_mode_at(&mut self, index: usize) {
        if let Some(mode) = Mode::ALL.get(index).copied() {
            self.start_game(mode);
        }
    }

    fn start_game(&mut self, mode: Mode) {
        self.screen = Screen::Game(Box::new(Game::new(mode)));
        self.result_recorded = false;
        self.new_best = false;
    }

    fn restart_game(&mut self) {
        let Some(game) = self.game_mut() else {
            return;
        };
        game.restart();
        self.result_recorded = false;
        self.new_best = false;
    }

    fn record_result_if_needed(&mut self) {
        if self.result_recorded {
            return;
        }
        let result = self.game().map(|game| {
            (
                game.mode(),
                game.status(),
                Record::new(
                    game.stats().score(),
                    game.stats().lines(),
                    game.stats().level(),
                    game.elapsed(),
                ),
            )
        });
        let Some((mode, status, record)) = result else {
            return;
        };
        if !matches!(status, Status::GameOver | Status::Finished) {
            return;
        }
        if mode != Mode::Sprint || status == Status::Finished {
            self.new_best = self.records.submit(mode, record);
        }
        self.result_recorded = true;
    }

    fn return_to_menu(&mut self) {
        let mode = match self.game() {
            Some(game) if game.status() != Status::Playing => Some(game.mode()),
            _ => None,
        };
        if let Some(mode) = mode {
            self.screen = Screen::Menu(ModeMenu::at(mode));
        }
    }

    fn apply_to_menu(&mut self, apply: impl FnOnce(&mut ModeMenu)) {
        if let Some(menu) = self.menu_mut() {
            apply(menu);
        }
    }

    fn apply_to_game(&mut self, apply: impl FnOnce(&mut Game)) {
        if let Some(game) = self.game_mut() {
            apply(game);
        }
    }

    fn menu(&self) -> Option<&ModeMenu> {
        match &self.screen {
            Screen::Menu(menu) => Some(menu),
            Screen::Game(_) => None,
        }
    }

    fn menu_mut(&mut self) -> Option<&mut ModeMenu> {
        match &mut self.screen {
            Screen::Menu(menu) => Some(menu),
            Screen::Game(_) => None,
        }
    }

    fn game(&self) -> Option<&Game> {
        match &self.screen {
            Screen::Game(game) => Some(game.as_ref()),
            Screen::Menu(_) => None,
        }
    }

    fn game_mut(&mut self) -> Option<&mut Game> {
        match &mut self.screen {
            Screen::Game(game) => Some(game.as_mut()),
            Screen::Menu(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn test_app() -> App {
        App::with_records(Records::in_memory())
    }

    fn selected_menu_mode(app: &App) -> Mode {
        app.menu().expect("app is on the menu").selected()
    }

    fn active_game(app: &App) -> &Game {
        app.game().expect("app is in a game")
    }

    #[test]
    fn enter_starts_the_selected_mode() {
        let mut app = test_app();
        app.handle_key(key(KeyCode::Down));
        assert_eq!(selected_menu_mode(&app), Mode::Sprint);
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(active_game(&app).mode(), Mode::Sprint);
    }

    #[test]
    fn number_keys_start_modes_directly() {
        let mut app = test_app();
        app.handle_key(key(KeyCode::Char('3')));
        assert_eq!(active_game(&app).mode(), Mode::Ultra);
    }

    #[test]
    fn menu_is_only_available_after_the_game_stops() {
        let mut app = test_app();
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('m')));
        assert!(app.game().is_some());
        app.handle_key(key(KeyCode::Char('p')));
        app.handle_key(key(KeyCode::Char('m')));
        assert_eq!(selected_menu_mode(&app), Mode::Marathon);
    }

    #[test]
    fn menu_is_available_after_a_finished_game() {
        let mut app = test_app();
        app.handle_key(key(KeyCode::Char('3')));
        app.apply_to_game(|game| game.update(crate::mode::ULTRA_TIME));
        app.handle_key(key(KeyCode::Char('m')));
        assert_eq!(selected_menu_mode(&app), Mode::Ultra);
    }

    #[test]
    fn finished_game_updates_personal_bests() {
        let mut app = test_app();
        app.handle_key(key(KeyCode::Char('3')));
        app.handle_key(key(KeyCode::Char(' ')));
        app.update(crate::mode::ULTRA_TIME);
        assert!(app.new_best);
        assert!(app.records.bests().best(Mode::Ultra).is_some());

        app.handle_key(key(KeyCode::Char('m')));
        app.handle_key(key(KeyCode::Char('3')));
        app.update(crate::mode::ULTRA_TIME);
        assert!(!app.new_best);
    }

    #[test]
    fn q_quits_from_the_menu() {
        let mut app = test_app();
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.quit);
    }
}

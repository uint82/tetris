use std::io;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};

use crate::daily::DailyDate;
use crate::game::{Action, Game, Status};
use crate::input::{self, MenuAction};
use crate::menu::{MenuChoice, ModeMenu};
use crate::mode::Mode;
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
            Screen::Menu(menu) => {
                ui::draw_menu(frame, menu, self.records.bests(), DailyDate::today())
            }
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
            MenuAction::Start => self.start_selected_choice(),
            MenuAction::StartChoice(index) => self.start_choice_at(index),
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

    fn start_selected_choice(&mut self) {
        if let Some(choice) = self.menu().map(ModeMenu::selected) {
            self.start_choice(choice);
        }
    }

    fn start_choice_at(&mut self, index: usize) {
        if let Some(choice) = MenuChoice::ALL.get(index).copied() {
            self.start_choice(choice);
        }
    }

    fn start_choice(&mut self, choice: MenuChoice) {
        let game = match choice {
            MenuChoice::Mode(mode) => Game::new(mode),
            MenuChoice::Daily => Game::daily(DailyDate::today()),
        };
        self.screen = Screen::Game(Box::new(game));
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
                game.daily_date(),
                game.status(),
                Record::new(
                    game.stats().score(),
                    game.stats().lines(),
                    game.stats().level(),
                    game.elapsed(),
                ),
            )
        });
        let Some((mode, daily, status, record)) = result else {
            return;
        };
        if !matches!(status, Status::GameOver | Status::Finished) {
            return;
        }
        self.new_best = self.store_result(mode, daily, status, record);
        self.result_recorded = true;
    }

    fn store_result(
        &mut self,
        mode: Mode,
        daily: Option<DailyDate>,
        status: Status,
        record: Record,
    ) -> bool {
        if let Some(date) = daily {
            return self.records.submit_daily(date, record);
        }
        if mode == Mode::Sprint && status != Status::Finished {
            return false;
        }
        self.records.submit(mode, record)
    }

    fn return_to_menu(&mut self) {
        let choice = match self.game() {
            Some(game) if game.status() != Status::Playing => Some(menu_choice_for(game)),
            _ => None,
        };
        if let Some(choice) = choice {
            self.screen = Screen::Menu(ModeMenu::at(choice));
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

fn menu_choice_for(game: &Game) -> MenuChoice {
    match game.daily_date() {
        Some(_) => MenuChoice::Daily,
        None => MenuChoice::Mode(game.mode()),
    }
}

#[cfg(test)]
mod tests;

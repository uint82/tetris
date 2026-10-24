use std::io;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};

use crate::game::{Action, Game};
use crate::{input, ui};

const FRAME_TIME: Duration = Duration::from_millis(16);
const MAX_FRAME_TIME: Duration = Duration::from_millis(100);

pub fn run() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

struct App {
    game: Game,
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            game: Game::new(),
            quit: false,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| ui::draw(frame, &self.game))?;
        while !self.quit {
            let frame_start = Instant::now();
            self.handle_events(FRAME_TIME)?;
            self.game.update(frame_start.elapsed().min(MAX_FRAME_TIME));
            if self.quit {
                break;
            }
            terminal.draw(|frame| ui::draw(frame, &self.game))?;
        }
        Ok(())
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
        if let Some(action) = input::action(key) {
            self.apply(action);
        }
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Quit => self.quit = true,
            Action::Restart => self.game.restart(),
            Action::TogglePause => self.game.toggle_pause(),
            action => self.game.apply(action),
        }
    }
}

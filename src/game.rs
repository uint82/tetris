use std::collections::VecDeque;
use std::time::Duration;

use crate::bag::Bag;
use crate::board::{Board, HIDDEN_ROWS};
use crate::countdown::Countdown;
use crate::effects::Effects;
use crate::kicks;
use crate::mode::Mode;
use crate::piece::{ActivePiece, PieceKind, Rotation};
use crate::scoring::{Clear, ScoreState};
use crate::tspin;

pub const NEXT_COUNT: usize = 5;
const LOCK_DELAY: Duration = Duration::from_millis(500);
const MAX_LOCK_RESETS: u8 = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Paused,
    GameOver,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    MoveLeft,
    MoveRight,
    RotateClockwise,
    RotateCounterClockwise,
    SoftDrop,
    HardDrop,
    Hold,
    TogglePause,
    Restart,
    Menu,
    Quit,
}

pub struct Game {
    board: Board,
    active: ActivePiece,
    bag: Bag,
    queue: VecDeque<PieceKind>,
    held: Option<PieceKind>,
    can_hold: bool,
    stats: ScoreState,
    mode: Mode,
    status: Status,
    elapsed: Duration,
    gravity: Duration,
    gravity_elapsed: Duration,
    lock_elapsed: Duration,
    lock_resets: u8,
    last_action_rotation: bool,
    effects: Effects,
    countdown: Option<Countdown>,
}

impl Game {
    pub fn new(mode: Mode) -> Self {
        let mut bag = Bag::new();
        let mut queue = VecDeque::with_capacity(NEXT_COUNT);
        for _ in 0..NEXT_COUNT {
            queue.push_back(bag.next());
        }
        let kind = queue.pop_front().expect("queue is prefilled");
        queue.push_back(bag.next());
        Self {
            board: Board::new(),
            active: ActivePiece::new(kind),
            bag,
            queue,
            held: None,
            can_hold: true,
            stats: ScoreState::new(),
            mode,
            status: Status::Playing,
            elapsed: Duration::ZERO,
            gravity: gravity_interval(1),
            gravity_elapsed: Duration::ZERO,
            lock_elapsed: Duration::ZERO,
            lock_resets: 0,
            last_action_rotation: false,
            effects: Effects::default(),
            countdown: mode.is_timed().then(Countdown::new),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        if self.status != Status::Playing {
            return;
        }
        if self.countdown.is_some() {
            self.tick_countdown(dt);
            return;
        }
        self.elapsed += dt;
        self.effects.update(dt);
        if self.finish_if_complete() {
            return;
        }
        self.apply_gravity(dt);
        self.apply_lock_delay(dt);
    }

    pub fn apply(&mut self, action: Action) {
        if self.status != Status::Playing || self.countdown.is_some() {
            return;
        }
        match action {
            Action::MoveLeft => self.shift(-1),
            Action::MoveRight => self.shift(1),
            Action::RotateClockwise => self.rotate(true),
            Action::RotateCounterClockwise => self.rotate(false),
            Action::SoftDrop => self.soft_drop(),
            Action::HardDrop => self.hard_drop(),
            Action::Hold => self.hold(),
            Action::TogglePause | Action::Restart | Action::Menu | Action::Quit => {}
        }
    }

    pub fn toggle_pause(&mut self) {
        self.status = match self.status {
            Status::Playing => Status::Paused,
            Status::Paused => Status::Playing,
            Status::GameOver | Status::Finished => return,
        };
    }

    pub fn restart(&mut self) {
        *self = Self::new(self.mode);
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn active(&self) -> ActivePiece {
        self.active
    }

    pub fn ghost(&self) -> ActivePiece {
        self.active.moved(0, self.drop_distance())
    }

    pub fn held(&self) -> Option<PieceKind> {
        self.held
    }

    pub fn can_hold(&self) -> bool {
        self.can_hold
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn stats(&self) -> &ScoreState {
        &self.stats
    }

    pub fn effects(&self) -> &Effects {
        &self.effects
    }

    pub fn countdown_number(&self) -> Option<u32> {
        self.countdown.as_ref().map(Countdown::number)
    }

    pub fn next(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.queue.iter().copied()
    }

    fn shift(&mut self, dx: i32) {
        let moved = self.active.moved(dx, 0);
        if self.board.fits(&moved) {
            self.active = moved;
            self.last_action_rotation = false;
            self.reset_lock_delay();
        }
    }

    fn rotate(&mut self, clockwise: bool) {
        let to = if clockwise {
            self.active.rotation.clockwise()
        } else {
            self.active.rotation.counter_clockwise()
        };
        if let Some(rotated) = self.kicked_rotation(to, clockwise) {
            self.active = rotated;
            self.last_action_rotation = true;
            self.reset_lock_delay();
        }
    }

    fn kicked_rotation(&self, to: Rotation, clockwise: bool) -> Option<ActivePiece> {
        let rotated = self.active.rotated(to);
        let offsets = kicks::offsets(self.active.kind, self.active.rotation, clockwise);
        offsets.iter().find_map(|&(dx, dy)| {
            let candidate = rotated.moved(dx, dy);
            self.board.fits(&candidate).then_some(candidate)
        })
    }

    fn soft_drop(&mut self) {
        if self.move_down() {
            self.stats.add_soft_drop(1);
            self.last_action_rotation = false;
        }
    }

    fn hard_drop(&mut self) {
        let distance = self.drop_distance();
        if distance > 0 {
            self.effects.start_hard_drop(self.active, distance);
            self.active = self.active.moved(0, distance);
            self.stats.add_hard_drop(distance as u32);
            self.last_action_rotation = false;
        }
        self.lock();
    }

    fn hold(&mut self) {
        if !self.can_hold {
            return;
        }
        let current = self.active.kind;
        match self.held.replace(current) {
            Some(kind) => self.spawn(kind),
            None => self.spawn_next(),
        }
        self.can_hold = false;
    }

    fn tick_countdown(&mut self, dt: Duration) {
        let Some(countdown) = &mut self.countdown else {
            return;
        };
        countdown.update(dt);
        if !countdown.is_active() {
            self.countdown = None;
        }
    }

    fn apply_gravity(&mut self, dt: Duration) {
        self.gravity_elapsed += dt;
        while self.gravity_elapsed >= self.gravity {
            self.gravity_elapsed -= self.gravity;
            if !self.move_down() {
                break;
            }
        }
    }

    fn apply_lock_delay(&mut self, dt: Duration) {
        if !self.is_grounded() {
            self.lock_elapsed = Duration::ZERO;
            return;
        }
        self.lock_elapsed += dt;
        if self.lock_elapsed >= LOCK_DELAY {
            self.lock();
        }
    }

    fn move_down(&mut self) -> bool {
        let moved = self.active.moved(0, 1);
        if !self.board.fits(&moved) {
            return false;
        }
        self.active = moved;
        true
    }

    fn drop_distance(&self) -> i32 {
        let mut distance = 0;
        while self.board.fits(&self.active.moved(0, distance + 1)) {
            distance += 1;
        }
        distance
    }

    fn reset_lock_delay(&mut self) {
        if self.lock_elapsed > Duration::ZERO && self.lock_resets < MAX_LOCK_RESETS {
            self.lock_elapsed = Duration::ZERO;
            self.lock_resets += 1;
        }
    }

    fn is_grounded(&self) -> bool {
        !self.board.fits(&self.active.moved(0, 1))
    }

    fn lock(&mut self) {
        if self
            .active
            .cells()
            .iter()
            .any(|&(_, y)| y < HIDDEN_ROWS as i32)
        {
            self.status = Status::GameOver;
            return;
        }
        self.board.lock(&self.active);
        let tspin = tspin::detect(&self.board, &self.active, self.last_action_rotation);
        let rows = self.board.full_rows();
        let lines = self.board.clear_lines();
        if rows != 0 {
            self.effects.start_line_clear(rows);
        }
        let perfect = self.board.is_empty();
        self.stats.register(Clear {
            lines,
            tspin,
            perfect,
        });
        self.gravity = gravity_interval(self.stats.level());
        self.can_hold = true;
        if self.finish_if_complete() {
            return;
        }
        self.spawn_next();
    }

    fn finish_if_complete(&mut self) -> bool {
        if !self.mode.objective_met(self.stats.lines(), self.elapsed) {
            return false;
        }
        self.status = Status::Finished;
        true
    }

    fn spawn_next(&mut self) {
        let kind = self.take_next();
        self.spawn(kind);
    }

    fn take_next(&mut self) -> PieceKind {
        let kind = self.queue.pop_front().expect("queue is never empty");
        self.queue.push_back(self.bag.next());
        kind
    }

    fn spawn(&mut self, kind: PieceKind) {
        self.active = ActivePiece::new(kind);
        self.gravity_elapsed = Duration::ZERO;
        self.lock_elapsed = Duration::ZERO;
        self.lock_resets = 0;
        self.last_action_rotation = false;
        if !self.board.fits(&self.active) {
            self.status = Status::GameOver;
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Mode::Marathon)
    }
}

pub fn gravity_interval(level: u32) -> Duration {
    let level = level.clamp(1, 20) as f64;
    let seconds = (0.8 - (level - 1.0) * 0.007).powf(level - 1.0);
    Duration::from_secs_f64(seconds.max(0.016))
}

#[cfg(test)]
mod tests;

use super::*;
use crate::board;
use crate::countdown::COUNTDOWN_TIME;
use crate::daily::DailyDate;
use crate::mode::{Mode, SPRINT_LINES, TIME_LIMIT};
use crate::piece::Position;
use crate::scoring::Clear;

fn new_game() -> Game {
    Game::new(Mode::Marathon)
}

fn skip_countdown(game: &mut Game) {
    game.update(COUNTDOWN_TIME);
}

fn opening(game: &Game) -> (PieceKind, Vec<PieceKind>) {
    (game.active().kind, game.next().collect())
}

fn filled_cells(game: &Game) -> usize {
    let mut count = 0;
    for x in 0..board::WIDTH {
        for y in 0..board::HEIGHT {
            if game.board().cell(x, y).is_some() {
                count += 1;
            }
        }
    }
    count
}

fn place_piece(game: &mut Game, kind: PieceKind, x: i32, y: i32) {
    game.active = ActivePiece {
        kind,
        rotation: Rotation::Spawn,
        position: Position { x, y },
    };
}

#[test]
fn daily_challenge_uses_a_stable_seed() {
    let date = DailyDate::from_days(20_751);
    let first = Game::daily(date);
    let second = Game::daily(date);
    assert_eq!(first.mode(), Mode::Time);
    assert_eq!(first.daily_date(), Some(date));
    assert_eq!(opening(&first), opening(&second));
    assert_ne!(
        opening(&first),
        opening(&Game::daily(DailyDate::from_days(20_752)))
    );
}

#[test]
fn daily_restart_preserves_seed_and_date() {
    let date = DailyDate::from_days(20_751);
    let mut game = Game::daily(date);
    let expected = opening(&game);
    game.restart();
    assert_eq!(game.mode(), Mode::Time);
    assert_eq!(game.daily_date(), Some(date));
    assert_eq!(game.countdown_number(), Some(3));
    assert_eq!(opening(&game), expected);
}

#[test]
fn classic_disables_hold_and_hard_drop() {
    let mut game = Game::new(Mode::Classic);
    let active = game.active();
    game.apply(Action::HardDrop);
    assert_eq!(game.active(), active);
    assert_eq!(filled_cells(&game), 0);
    game.apply(Action::Hold);
    assert_eq!(game.held(), None);
}

#[test]
fn classic_soft_drop_locks_on_contact() {
    let mut game = Game::new(Mode::Classic);
    place_piece(&mut game, PieceKind::O, 4, board::HEIGHT as i32 - 2);
    game.apply(Action::SoftDrop);
    assert_eq!(filled_cells(&game), 4);
}

#[test]
fn classic_gravity_locks_on_contact() {
    let mut game = Game::new(Mode::Classic);
    place_piece(&mut game, PieceKind::O, 4, board::HEIGHT as i32 - 2);
    game.update(Duration::from_millis(500));
    assert_eq!(filled_cells(&game), 0);
    game.update(Duration::from_millis(300));
    assert_eq!(filled_cells(&game), 4);
}

#[test]
fn classic_rotation_has_no_wall_kicks() {
    let mut classic = Game::new(Mode::Classic);
    place_piece(&mut classic, PieceKind::T, 3, 5);
    classic.board.set(4, 7, PieceKind::I);
    let original = classic.active();
    classic.apply(Action::RotateClockwise);
    assert_eq!(classic.active(), original);

    let mut marathon = Game::new(Mode::Marathon);
    place_piece(&mut marathon, PieceKind::T, 3, 5);
    marathon.board.set(4, 7, PieceKind::I);
    marathon.apply(Action::RotateClockwise);
    assert_ne!(marathon.active(), original);
}

#[test]
fn classic_gravity_starts_slower_than_marathon() {
    assert_eq!(classic_gravity_interval(1), Duration::from_millis(800));
    assert_eq!(classic_gravity_interval(10), Duration::from_millis(100));
}

#[test]
fn timed_modes_start_with_a_countdown() {
    assert_eq!(Game::new(Mode::Sprint).countdown_number(), Some(3));
    assert_eq!(Game::new(Mode::Time).countdown_number(), Some(3));
    assert_eq!(Game::new(Mode::Marathon).countdown_number(), None);
}

#[test]
fn countdown_blocks_input_and_the_clock() {
    let mut game = Game::new(Mode::Time);
    game.apply(Action::HardDrop);
    assert_eq!(filled_cells(&game), 0);
    game.update(Duration::from_secs(1));
    assert_eq!(game.countdown_number(), Some(2));
    assert_eq!(game.elapsed(), Duration::ZERO);
    assert_eq!(game.status(), Status::Playing);
}

#[test]
fn pause_freezes_the_countdown() {
    let mut game = Game::new(Mode::Time);
    game.update(Duration::from_secs(1));
    game.toggle_pause();
    game.update(Duration::from_secs(1));
    assert_eq!(game.countdown_number(), Some(2));
    game.toggle_pause();
    game.update(Duration::from_secs(1));
    assert_eq!(game.countdown_number(), Some(1));
}

#[test]
fn countdown_releases_the_clock() {
    let mut game = Game::new(Mode::Time);
    skip_countdown(&mut game);
    assert_eq!(game.countdown_number(), None);
    game.update(Duration::from_secs(1));
    assert_eq!(game.elapsed(), Duration::from_secs(1));
}

#[test]
fn new_game_has_a_full_preview_queue() {
    let game = new_game();
    assert_eq!(game.status(), Status::Playing);
    assert_eq!(game.next().count(), NEXT_COUNT);
    assert!(game.board().fits(&game.active()));
}

#[test]
fn hard_drop_locks_the_active_piece() {
    let mut game = new_game();
    let expected_next = game.next().next().expect("queue has pieces");
    game.apply(Action::HardDrop);
    assert_eq!(game.active().kind, expected_next);
    assert_eq!(filled_cells(&game), 4);
    assert!(game.stats().score() > 0);
    assert!(game.effects().hard_drop().is_some());
}

#[test]
fn clearing_a_line_starts_a_flash() {
    let mut game = new_game();
    for x in 0..board::WIDTH {
        if x != 4 && x != 5 {
            game.board.set(x, board::HEIGHT - 1, PieceKind::I);
        }
    }
    place_piece(&mut game, PieceKind::O, 4, 2);
    game.apply(Action::HardDrop);
    assert_eq!(game.stats().lines(), 1);
    let flash = game.effects().line_clear().expect("line clear effect");
    assert!(flash.contains(board::HEIGHT - 1));
}

#[test]
fn hold_swaps_once_per_piece() {
    let mut game = new_game();
    let first = game.active().kind;
    game.apply(Action::Hold);
    assert_eq!(game.held(), Some(first));
    assert!(!game.can_hold());
    let active_after_hold = game.active().kind;
    game.apply(Action::Hold);
    assert_eq!(game.active().kind, active_after_hold);
}

#[test]
fn hold_is_available_again_after_locking() {
    let mut game = new_game();
    game.apply(Action::Hold);
    game.apply(Action::HardDrop);
    assert!(game.can_hold());
}

#[test]
fn pause_stops_the_game() {
    let mut game = new_game();
    game.toggle_pause();
    assert_eq!(game.status(), Status::Paused);
    game.apply(Action::HardDrop);
    assert_eq!(filled_cells(&game), 0);
    game.toggle_pause();
    assert_eq!(game.status(), Status::Playing);
}

#[test]
fn restart_clears_the_board() {
    let mut game = new_game();
    game.apply(Action::HardDrop);
    game.restart();
    assert_eq!(filled_cells(&game), 0);
    assert_eq!(game.stats().score(), 0);
    assert_eq!(game.held(), None);
}

#[test]
fn spawning_into_an_occupied_cell_ends_the_game() {
    let mut game = new_game();
    for (x, y) in ActivePiece::new(PieceKind::T).cells() {
        game.board.set(x as usize, y as usize, PieceKind::I);
    }
    game.spawn(PieceKind::T);
    assert_eq!(game.status(), Status::GameOver);
}

#[test]
fn gravity_moves_the_piece_down() {
    let mut game = new_game();
    let start_y = game.active().position.y;
    game.update(Duration::from_secs(1));
    assert!(game.active().position.y > start_y);
}

#[test]
fn grounded_piece_locks_after_the_delay() {
    let mut game = new_game();
    place_piece(&mut game, PieceKind::O, 4, board::HEIGHT as i32 - 2);
    game.update(LOCK_DELAY);
    assert_eq!(filled_cells(&game), 4);
}

#[test]
fn movement_resets_the_lock_delay() {
    let mut game = new_game();
    place_piece(&mut game, PieceKind::O, 4, board::HEIGHT as i32 - 2);
    game.update(Duration::from_millis(400));
    game.apply(Action::MoveLeft);
    game.update(Duration::from_millis(400));
    assert_eq!(filled_cells(&game), 0);
    game.update(Duration::from_millis(200));
    assert_eq!(filled_cells(&game), 4);
}

#[test]
fn locking_above_the_visible_field_ends_the_game() {
    let mut game = new_game();
    place_piece(&mut game, PieceKind::O, 4, 0);
    game.lock();
    assert_eq!(game.status(), Status::GameOver);
}

#[test]
fn sprint_finishes_at_the_line_goal() {
    let mut game = Game::new(Mode::Sprint);
    skip_countdown(&mut game);
    for _ in 0..10 {
        game.stats.register(Clear {
            lines: 4,
            tspin: None,
            perfect: false,
        });
    }
    game.update(Duration::ZERO);
    assert_eq!(game.stats().lines(), SPRINT_LINES);
    assert_eq!(game.status(), Status::Finished);
}

#[test]
fn time_finishes_at_the_time_limit() {
    let mut game = Game::new(Mode::Time);
    skip_countdown(&mut game);
    game.update(TIME_LIMIT);
    assert_eq!(game.status(), Status::Finished);
    game.toggle_pause();
    assert_eq!(game.status(), Status::Finished);
}

#[test]
fn pause_stops_the_clock() {
    let mut game = new_game();
    game.toggle_pause();
    game.update(Duration::from_secs(1));
    assert_eq!(game.elapsed(), Duration::ZERO);
}

#[test]
fn restart_preserves_the_mode() {
    let mut game = Game::new(Mode::Sprint);
    game.restart();
    assert_eq!(game.mode(), Mode::Sprint);
    assert_eq!(game.status(), Status::Playing);
}

#[test]
fn gravity_interval_starts_at_one_second() {
    assert_eq!(gravity_interval(1), Duration::from_secs(1));
}

#[test]
fn gravity_interval_has_a_lower_bound() {
    assert!(gravity_interval(1000) >= Duration::from_millis(16));
}

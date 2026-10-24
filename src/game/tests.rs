use super::*;
use crate::board;
use crate::piece::Position;

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
fn new_game_has_a_full_preview_queue() {
    let game = Game::new();
    assert_eq!(game.status(), Status::Playing);
    assert_eq!(game.next().count(), NEXT_COUNT);
    assert!(game.board().fits(&game.active()));
}

#[test]
fn hard_drop_locks_the_active_piece() {
    let mut game = Game::new();
    let expected_next = game.next().next().expect("queue has pieces");
    game.apply(Action::HardDrop);
    assert_eq!(game.active().kind, expected_next);
    assert_eq!(filled_cells(&game), 4);
    assert!(game.stats().score() > 0);
}

#[test]
fn hold_swaps_once_per_piece() {
    let mut game = Game::new();
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
    let mut game = Game::new();
    game.apply(Action::Hold);
    game.apply(Action::HardDrop);
    assert!(game.can_hold());
}

#[test]
fn pause_stops_the_game() {
    let mut game = Game::new();
    game.toggle_pause();
    assert_eq!(game.status(), Status::Paused);
    game.apply(Action::HardDrop);
    assert_eq!(filled_cells(&game), 0);
    game.toggle_pause();
    assert_eq!(game.status(), Status::Playing);
}

#[test]
fn restart_clears_the_board() {
    let mut game = Game::new();
    game.apply(Action::HardDrop);
    game.restart();
    assert_eq!(filled_cells(&game), 0);
    assert_eq!(game.stats().score(), 0);
    assert_eq!(game.held(), None);
}

#[test]
fn spawning_into_an_occupied_cell_ends_the_game() {
    let mut game = Game::new();
    for (x, y) in ActivePiece::new(PieceKind::T).cells() {
        game.board.set(x as usize, y as usize, PieceKind::I);
    }
    game.spawn(PieceKind::T);
    assert_eq!(game.status(), Status::GameOver);
}

#[test]
fn gravity_moves_the_piece_down() {
    let mut game = Game::new();
    let start_y = game.active().position.y;
    game.update(Duration::from_secs(1));
    assert!(game.active().position.y > start_y);
}

#[test]
fn grounded_piece_locks_after_the_delay() {
    let mut game = Game::new();
    place_piece(&mut game, PieceKind::O, 4, board::HEIGHT as i32 - 2);
    game.update(LOCK_DELAY);
    assert_eq!(filled_cells(&game), 4);
}

#[test]
fn movement_resets_the_lock_delay() {
    let mut game = Game::new();
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
    let mut game = Game::new();
    place_piece(&mut game, PieceKind::O, 4, 0);
    game.lock();
    assert_eq!(game.status(), Status::GameOver);
}

#[test]
fn gravity_interval_starts_at_one_second() {
    assert_eq!(gravity_interval(1), Duration::from_secs(1));
}

#[test]
fn gravity_interval_has_a_lower_bound() {
    assert!(gravity_interval(1000) >= Duration::from_millis(16));
}

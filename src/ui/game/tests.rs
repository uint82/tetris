use super::*;
use crate::countdown::COUNTDOWN_TIME;
use crate::daily::DailyDate;
use crate::game::Action;
use crate::mode::ULTRA_TIME;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn draws_every_mode() {
    for mode in Mode::ALL {
        let backend = TestBackend::new(TOTAL_WIDTH, TOTAL_HEIGHT);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let game = Game::new(mode);
        let bests = PersonalBests::default();
        terminal
            .draw(|frame| draw(frame, &game, &bests, false))
            .expect("game renders");
    }
}

#[test]
fn effects_render_on_the_board() {
    let board = Board::new();
    let active = ActivePiece::new(PieceKind::T);
    let ghost = active;
    let mut effects = Effects::default();
    effects.start_line_clear(1 << HIDDEN_ROWS);
    let flash = board_span(&board, &active, &ghost, &effects, 4, HIDDEN_ROWS);
    assert_eq!(flash.content, "██");
    assert_eq!(flash.style, theme::line_flash(true));

    let mut effects = Effects::default();
    effects.start_hard_drop(ActivePiece::new(PieceKind::O), 3);
    let trail = board_span(&board, &active, &ghost, &effects, 4, HIDDEN_ROWS + 3);
    assert_eq!(trail.content, "██");
    assert_eq!(trail.style, block_span(PieceKind::O, false).style);
}

#[test]
fn draws_hard_drop_trail() {
    let backend = TestBackend::new(TOTAL_WIDTH, TOTAL_HEIGHT);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let mut game = Game::new(Mode::Marathon);
    let bests = PersonalBests::default();
    game.apply(Action::HardDrop);
    terminal
        .draw(|frame| draw(frame, &game, &bests, false))
        .expect("effects render");
}

#[test]
fn draws_a_small_terminal_without_panicking() {
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let game = Game::new(Mode::Marathon);
    let bests = PersonalBests::default();
    terminal
        .draw(|frame| draw(frame, &game, &bests, false))
        .expect("game renders");
}

#[test]
fn draws_the_pause_overlay() {
    let backend = TestBackend::new(TOTAL_WIDTH, TOTAL_HEIGHT);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let mut game = Game::new(Mode::Marathon);
    let bests = PersonalBests::default();
    game.toggle_pause();
    terminal
        .draw(|frame| draw(frame, &game, &bests, false))
        .expect("pause overlay renders");
}

#[test]
fn draws_the_daily_challenge_and_result() {
    let backend = TestBackend::new(TOTAL_WIDTH, TOTAL_HEIGHT);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let mut game = Game::daily(DailyDate::from_days(20_751));
    let bests = PersonalBests::default();
    terminal
        .draw(|frame| draw(frame, &game, &bests, false))
        .expect("daily countdown renders");
    game.update(COUNTDOWN_TIME);
    game.update(ULTRA_TIME);
    terminal
        .draw(|frame| draw(frame, &game, &bests, true))
        .expect("daily result renders");
}

#[test]
fn draws_the_finished_overlay() {
    let backend = TestBackend::new(TOTAL_WIDTH, TOTAL_HEIGHT);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let mut game = Game::new(Mode::Ultra);
    let bests = PersonalBests::default();
    game.update(COUNTDOWN_TIME);
    game.update(ULTRA_TIME);
    terminal
        .draw(|frame| draw(frame, &game, &bests, true))
        .expect("finished overlay renders");
}

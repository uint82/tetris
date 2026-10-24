use super::*;
use crate::countdown::COUNTDOWN_TIME;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn test_app() -> App {
    App::with_records(Records::in_memory())
}

fn skip_countdown(app: &mut App) {
    app.apply_to_game(|game| game.update(COUNTDOWN_TIME));
}

fn selected_menu_choice(app: &App) -> MenuChoice {
    app.menu().expect("app is on the menu").selected()
}

fn active_game(app: &App) -> &Game {
    app.game().expect("app is in a game")
}

#[test]
fn enter_starts_the_selected_mode() {
    let mut app = test_app();
    app.handle_key(key(KeyCode::Down));
    assert_eq!(selected_menu_choice(&app), MenuChoice::Mode(Mode::Sprint));
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
    assert_eq!(selected_menu_choice(&app), MenuChoice::Mode(Mode::Marathon));
}

#[test]
fn menu_is_available_after_a_finished_game() {
    let mut app = test_app();
    app.handle_key(key(KeyCode::Char('3')));
    skip_countdown(&mut app);
    app.apply_to_game(|game| game.update(crate::mode::ULTRA_TIME));
    app.handle_key(key(KeyCode::Char('m')));
    assert_eq!(selected_menu_choice(&app), MenuChoice::Mode(Mode::Ultra));
}

#[test]
fn finished_game_updates_personal_bests() {
    let mut app = test_app();
    app.handle_key(key(KeyCode::Char('3')));
    skip_countdown(&mut app);
    app.handle_key(key(KeyCode::Char(' ')));
    app.update(crate::mode::ULTRA_TIME);
    assert!(app.new_best);
    assert!(app.records.bests().best(Mode::Ultra).is_some());

    app.handle_key(key(KeyCode::Char('m')));
    app.handle_key(key(KeyCode::Char('3')));
    skip_countdown(&mut app);
    app.update(crate::mode::ULTRA_TIME);
    assert!(!app.new_best);
}

#[test]
fn daily_choice_starts_a_seeded_challenge() {
    let mut app = test_app();
    for _ in 0..3 {
        app.handle_key(key(KeyCode::Down));
    }
    assert_eq!(selected_menu_choice(&app), MenuChoice::Daily);
    app.handle_key(key(KeyCode::Enter));
    let game = active_game(&app);
    assert_eq!(game.mode(), Mode::Ultra);
    assert_eq!(game.daily_date(), Some(DailyDate::today()));
}

#[test]
fn number_four_starts_the_daily_challenge() {
    let mut app = test_app();
    app.handle_key(key(KeyCode::Char('4')));
    assert!(active_game(&app).daily_date().is_some());
}

#[test]
fn daily_result_updates_only_the_daily_best() {
    let mut app = test_app();
    app.handle_key(key(KeyCode::Char('4')));
    skip_countdown(&mut app);
    app.handle_key(key(KeyCode::Char(' ')));
    app.update(crate::mode::ULTRA_TIME);
    assert!(app.new_best);
    assert!(app.records.bests().daily(DailyDate::today()).is_some());
    assert!(app.records.bests().best(Mode::Ultra).is_none());
}

#[test]
fn daily_game_returns_to_the_daily_menu_entry() {
    let mut app = test_app();
    app.handle_key(key(KeyCode::Char('4')));
    skip_countdown(&mut app);
    app.apply_to_game(|game| game.update(crate::mode::ULTRA_TIME));
    app.handle_key(key(KeyCode::Char('m')));
    assert_eq!(selected_menu_choice(&app), MenuChoice::Daily);
}

#[test]
fn q_quits_from_the_menu() {
    let mut app = test_app();
    app.handle_key(key(KeyCode::Char('q')));
    assert!(app.quit);
}

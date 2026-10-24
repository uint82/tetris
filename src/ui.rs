use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use crate::board::{Board, HEIGHT, HIDDEN_ROWS, VISIBLE_HEIGHT, WIDTH};
use crate::game::{Game, NEXT_COUNT, Status};
use crate::piece::{ActivePiece, PieceKind, Rotation};
use crate::scoring::ScoreState;
use crate::theme;

const BOARD_WIDTH: u16 = WIDTH as u16 * 2 + 2;
const BOARD_HEIGHT: u16 = VISIBLE_HEIGHT as u16 + 2;
const SIDE_WIDTH: u16 = 20;
const HOLD_HEIGHT: u16 = 4;
const NEXT_HEIGHT: u16 = 12;
const STATS_HEIGHT: u16 = 6;
const TOTAL_WIDTH: u16 = BOARD_WIDTH + SIDE_WIDTH;
const TOTAL_HEIGHT: u16 = BOARD_HEIGHT + 2;
const PREVIEW_COLUMNS: usize = 4;
const PREVIEW_ROWS: usize = 2;
const OVERLAY_WIDTH: u16 = 34;
const OVERLAY_HEIGHT: u16 = 8;
const STAT_LABEL_WIDTH: usize = 7;
const STAT_VALUE_WIDTH: usize = 11;

struct ScreenLayout {
    board: Rect,
    hold: Rect,
    next: Rect,
    stats: Rect,
    help: Rect,
    main: Rect,
}

pub fn draw(frame: &mut Frame, game: &Game) {
    let area = frame.area();
    if area.width < TOTAL_WIDTH || area.height < TOTAL_HEIGHT {
        render_too_small(frame, area);
        return;
    }
    let layout = screen_layout(area);
    render_board(frame, layout.board, game);
    render_hold(frame, layout.hold, game);
    render_next(frame, layout.next, game);
    render_stats(frame, layout.stats, game);
    render_help(frame, layout.help);
    render_overlay(frame, layout.main, game);
}

fn screen_layout(area: Rect) -> ScreenLayout {
    let main = area.centered(
        Constraint::Length(TOTAL_WIDTH),
        Constraint::Length(TOTAL_HEIGHT),
    );
    let side_x = main.x + BOARD_WIDTH;
    ScreenLayout {
        board: Rect::new(main.x, main.y, BOARD_WIDTH, BOARD_HEIGHT),
        hold: Rect::new(side_x, main.y, SIDE_WIDTH, HOLD_HEIGHT),
        next: Rect::new(side_x, main.y + HOLD_HEIGHT, SIDE_WIDTH, NEXT_HEIGHT),
        stats: Rect::new(
            side_x,
            main.y + HOLD_HEIGHT + NEXT_HEIGHT,
            SIDE_WIDTH,
            STATS_HEIGHT,
        ),
        help: Rect::new(main.x, main.y + BOARD_HEIGHT, TOTAL_WIDTH, 2),
        main,
    }
}

fn render_board(frame: &mut Frame, area: Rect, game: &Game) {
    let block = panel_block(" TETRIS ");
    frame.render_widget(Paragraph::new(board_lines(game)).block(block), area);
}

fn board_lines(game: &Game) -> Vec<Line<'static>> {
    let active = game.active();
    let ghost = game.ghost();
    (HIDDEN_ROWS..HEIGHT)
        .map(|y| {
            let spans = (0..WIDTH)
                .map(|x| board_span(game.board(), &active, &ghost, x, y))
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

fn board_span(
    board: &Board,
    active: &ActivePiece,
    ghost: &ActivePiece,
    x: usize,
    y: usize,
) -> Span<'static> {
    if active.occupies(x as i32, y as i32) {
        return block_span(active.kind, true);
    }
    if let Some(kind) = board.cell(x, y) {
        return block_span(kind, true);
    }
    if ghost.occupies(x as i32, y as i32) {
        return ghost_span();
    }
    empty_span()
}

fn render_hold(frame: &mut Frame, area: Rect, game: &Game) {
    let lines = match game.held() {
        Some(kind) => preview_lines(kind, game.can_hold()),
        None => blank_preview(),
    };
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(panel_block(" HOLD ")),
        area,
    );
}

fn render_next(frame: &mut Frame, area: Rect, game: &Game) {
    let mut lines = Vec::with_capacity(NEXT_COUNT * PREVIEW_ROWS);
    for kind in game.next() {
        lines.extend(preview_lines(kind, true));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(panel_block(" NEXT ")),
        area,
    );
}

fn render_stats(frame: &mut Frame, area: Rect, game: &Game) {
    let stats = game.stats();
    let lines = vec![
        stat_line("Score", stats.score().to_string()),
        stat_line("Level", stats.level().to_string()),
        stat_line("Lines", stats.lines().to_string()),
        combo_line(stats),
    ];
    frame.render_widget(Paragraph::new(lines).block(panel_block(" STATS ")), area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from("Arrows move   Down soft drop   X/Z rotate").style(theme::help()),
        Line::from("Space hard drop  C hold  P pause  Q quit").style(theme::help()),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn render_overlay(frame: &mut Frame, area: Rect, game: &Game) {
    match game.status() {
        Status::Playing => {}
        Status::Paused => {
            render_popup(frame, area, " PAUSED ", theme::pause_color(), pause_lines())
        }
        Status::GameOver => render_popup(
            frame,
            area,
            " GAME OVER ",
            theme::game_over_color(),
            game_over_lines(game.stats()),
        ),
    }
}

fn render_popup(
    frame: &mut Frame,
    area: Rect,
    title: &'static str,
    color: ratatui::style::Color,
    lines: Vec<Line<'static>>,
) {
    let popup = area.centered(
        Constraint::Length(OVERLAY_WIDTH),
        Constraint::Length(OVERLAY_HEIGHT),
    );
    frame.render_widget(Clear, popup);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(color))
        .title(Line::from(title).style(Style::new().fg(color).add_modifier(Modifier::BOLD)));
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(block),
        popup,
    );
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
    let width = TOTAL_WIDTH.min(area.width);
    let popup = area.centered(Constraint::Length(width), Constraint::Length(2));
    let lines = vec![
        Line::from("Terminal too small").style(theme::value()),
        Line::from(format!("Required size: {TOTAL_WIDTH} x {TOTAL_HEIGHT}")).style(theme::help()),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), popup);
}

fn panel_block(title: &'static str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::border())
        .title(Line::from(title).style(theme::title()))
}

fn preview_lines(kind: PieceKind, enabled: bool) -> Vec<Line<'static>> {
    let cells = kind.cells(Rotation::Spawn);
    let (min_x, max_x, min_y, max_y) = bounds(&cells);
    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;
    let pad_x = (PREVIEW_COLUMNS - width).div_ceil(2);
    let pad_y = (PREVIEW_ROWS - height).div_ceil(2);
    (0..PREVIEW_ROWS)
        .map(|row| {
            let spans = (0..PREVIEW_COLUMNS)
                .map(|column| {
                    let x = column as i32 - pad_x as i32 + min_x;
                    let y = row as i32 - pad_y as i32 + min_y;
                    if cells.contains(&(x, y)) {
                        block_span(kind, enabled)
                    } else {
                        empty_span()
                    }
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

fn blank_preview() -> Vec<Line<'static>> {
    (0..PREVIEW_ROWS).map(|_| Line::default()).collect()
}

fn bounds(cells: &[(i32, i32); 4]) -> (i32, i32, i32, i32) {
    let min_x = cells.iter().map(|cell| cell.0).min().unwrap_or(0);
    let max_x = cells.iter().map(|cell| cell.0).max().unwrap_or(0);
    let min_y = cells.iter().map(|cell| cell.1).min().unwrap_or(0);
    let max_y = cells.iter().map(|cell| cell.1).max().unwrap_or(0);
    (min_x, max_x, min_y, max_y)
}

fn stat_line(label: &str, value: String) -> Line<'static> {
    let label = format!("{label:<width$}", width = STAT_LABEL_WIDTH);
    let value = format!("{value:>width$}", width = STAT_VALUE_WIDTH);
    Line::from(vec![
        Span::styled(label, theme::label()),
        Span::styled(value, theme::value()),
    ])
}

fn combo_line(stats: &ScoreState) -> Line<'static> {
    if !stats.back_to_back() {
        return stat_line("Combo", stats.combo().max(0).to_string());
    }
    Line::from(vec![
        Span::styled(
            format!("{:<width$}", "Combo", width = STAT_LABEL_WIDTH),
            theme::label(),
        ),
        Span::styled(format!("{:>7}", stats.combo().max(0)), theme::value()),
        Span::styled(" B2B", theme::accent()),
    ])
}

fn block_span(kind: PieceKind, enabled: bool) -> Span<'static> {
    let mut style = Style::new().fg(theme::piece_color(kind));
    if !enabled {
        style = style.add_modifier(Modifier::DIM);
    }
    Span::styled("██", style)
}

fn ghost_span() -> Span<'static> {
    Span::styled("░░", theme::ghost())
}

fn empty_span() -> Span<'static> {
    Span::raw("  ")
}

fn pause_lines() -> Vec<Line<'static>> {
    vec![
        Line::default(),
        Line::default(),
        Line::from("P / Esc   Resume"),
        Line::from("Q         Quit"),
        Line::default(),
        Line::default(),
    ]
}

fn game_over_lines(stats: &ScoreState) -> Vec<Line<'static>> {
    vec![
        Line::default(),
        Line::from(format!("Score  {}", stats.score())),
        Line::from(format!(
            "Level  {}    Lines  {}",
            stats.level(),
            stats.lines()
        )),
        Line::default(),
        Line::from("Enter / R   Restart"),
        Line::from("Q           Quit"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn draws_a_new_game() {
        let backend = TestBackend::new(TOTAL_WIDTH, TOTAL_HEIGHT);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let game = Game::new();
        terminal
            .draw(|frame| draw(frame, &game))
            .expect("game renders");
    }

    #[test]
    fn draws_a_small_terminal_without_panicking() {
        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let game = Game::new();
        terminal
            .draw(|frame| draw(frame, &game))
            .expect("game renders");
    }

    #[test]
    fn draws_the_pause_overlay() {
        let backend = TestBackend::new(TOTAL_WIDTH, TOTAL_HEIGHT);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut game = Game::new();
        game.toggle_pause();
        terminal
            .draw(|frame| draw(frame, &game))
            .expect("pause overlay renders");
    }
}

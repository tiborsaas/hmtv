pub mod ascii;
pub mod generative;
mod screen1;
mod screen2;
mod screen3;
mod screen4;
pub mod theme;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        1 => screen1::render(frame, app),
        2 => screen2::render(frame, app),
        3 => screen3::render(frame, app),
        4 => screen4::render(frame, app),
        _ => screen2::render(frame, app),
    }
}

/// Keys shown in the footer keybind bar, as `(key, label)` pairs.
pub const KEYBINDS: &[(&str, &str)] = &[
    ("1-4", "SCREENS"),
    ("SPACE", "PAUSE"),
    ("+/-", "VOLUME"),
    ("R", "RESYNC"),
    ("T", "THEME"),
    ("Q", "QUIT"),
];

/// A single-line horizontal rule with a centered, theme-colored title, used
/// as a section header on screens 2-4. Renders into a `Constraint::Length(1)`
/// area.
pub fn header_rule<'a>(theme: Theme, title: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(theme.dim()))
        .title(
            Line::from(format!(" {title} ").fg(theme.accent()).bold()).alignment(Alignment::Center),
        )
}

/// Renders the shared footer as a row of individually bordered keybind boxes,
/// e.g. `[ 1-4 / SCREENS ]`. Expects a `Constraint::Length(4)` area.
pub fn render_keybind_bar(frame: &mut Frame, area: Rect, theme: Theme) {
    if area.width == 0 || area.height == 0 || KEYBINDS.is_empty() {
        return;
    }
    let constraints: Vec<Constraint> = KEYBINDS
        .iter()
        .map(|_| Constraint::Ratio(1, KEYBINDS.len() as u32))
        .collect();
    let boxes = Layout::horizontal(constraints).split(area);
    for (rect, (key, label)) in boxes.iter().zip(KEYBINDS.iter()) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme.dim()));
        let inner = block.inner(*rect);
        frame.render_widget(block, *rect);
        // The theme key shows the active theme's name so cycling through
        // themes has an immediate, visible confirmation.
        let shown_label = if *key == "T" { theme.label() } else { label };
        let lines = vec![
            Line::from(key.to_string().bold().fg(theme.accent())).alignment(Alignment::Center),
            Line::from(shown_label.to_string().fg(theme.dim())).alignment(Alignment::Center),
        ];
        frame.render_widget(Paragraph::new(lines), inner);
    }
}

/// Renders the `Action::Error` banner, if any, as a single dismissible-looking
/// line. Screens call this last so it overlays near the bottom.
pub fn error_banner(msg: &str) -> Line<'static> {
    Line::from(vec![
        " ERROR ".black().on_red().bold(),
        format!(" {msg}").red(),
    ])
}

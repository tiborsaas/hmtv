use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

use crate::app::App;
use crate::ui::ascii::format_mmss;
use crate::ui::{header_rule, render_keybind_bar};

/// Screen 2: Standard. Redesigned with a split layout:
/// - Left top: Music controls (numeric volume).
/// - Left bottom: About info.
/// - Right: Full-height playback history log.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = app.theme;
    let area = frame.area();

    let [header, main, keybinds] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(4),
    ])
    .areas(area);

    frame.render_widget(header_rule(theme, "HUMANMUSIC.TV · INFO"), header);

    let [left_col, right_col] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(main);

    let [controls_area, info_area] =
        Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(left_col);

    render_controls_panel(frame, app, controls_area);
    render_info_panel(frame, app, info_area);
    render_history_panel(frame, app, right_col);

    render_keybind_bar(frame, keybinds, theme);
}

fn render_controls_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.dim()))
        .title(" CONTROLS ");

    let inner = block.inner(area);
    let inner = Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(2),
    };
    frame.render_widget(block, area);

    let [title_line, artist_line, progress_line, _, vol_line] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    if let Some(np) = &app.now_playing {
        let t = &np.data.current_track;
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                "Title: ".fg(theme.dim()),
                t.title.clone().bold().fg(theme.fg()),
            ])),
            title_line,
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                "Artist: ".fg(theme.dim()),
                t.artist.clone().fg(theme.accent()),
            ])),
            artist_line,
        );

        let ratio = app.progress_ratio();
        let label = format!(
            "{} / {}",
            format_mmss(app.elapsed_secs()),
            format_mmss(app.duration_secs())
        );
        let gauge = Gauge::default()
            .gauge_style(Style::new().fg(theme.accent()).bg(theme.dim()))
            .ratio(ratio)
            .label(label);
        frame.render_widget(gauge, progress_line, );
    } else {
        frame.render_widget(Paragraph::new("Connecting...".fg(theme.dim())), title_line);
    }

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            "Volume: ".fg(theme.dim()),
            format!("{:.0}%", app.player_status.volume)
                .bold()
                .fg(theme.accent()),
        ])),
        vol_line,
    );
}

fn render_info_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.dim()))
        .title(" ABOUT ");

    let inner = block.inner(area);
    let inner = Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(2),
    };
    frame.render_widget(block, area);

    let text = vec![
        Line::from("HUMAN MUSIC TV".bold().fg(theme.accent())),
        Line::from(""),
        Line::from("A 24/7 curated stream of high-vibe music".fg(theme.fg())),
        Line::from("built for a better internet.".fg(theme.fg())),
        Line::from(""),
        Line::from(vec![
            "Website: ".fg(theme.dim()),
            "https://humanmusic.tv".fg(theme.fg()),
        ]),
        Line::from(vec![
            "GitHub:  ".fg(theme.dim()),
            "github.com/tiborsaas/hmtv".fg(theme.fg()),
        ]),
        Line::from(""),
        Line::from(vec![
            "Created by ".fg(theme.dim()),
            "tiborsaas".bold().fg(theme.accent()),
        ]),
    ];

    frame.render_widget(Paragraph::new(text), inner);
}

fn render_history_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.dim()))
        .title(" RECENT TRACKS ");

    let inner = block.inner(area);
    let inner = Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(2),
    };
    frame.render_widget(block, area);

    let mut items = Vec::new();
    for t in &app.history {
        items.push(Line::from(vec![
            format!("{:<20} ", t.artist).fg(theme.accent()),
            " — ".fg(theme.dim()),
            t.title.clone().fg(theme.fg()),
        ]));
    }

    if items.is_empty() {
        frame.render_widget(
            Paragraph::new("Waiting for tracks...".fg(theme.dim())),
            inner,
        );
    } else {
        frame.render_widget(Paragraph::new(items), inner);
    }
}

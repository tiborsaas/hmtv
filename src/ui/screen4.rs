use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph};

use crate::app::App;
use crate::ui::ascii::{ANTENNA, HMTV_LOGO, bars_line, format_mmss};
use crate::ui::{error_banner, footer_keybinds};

/// Screen 4: Full. Everything in Rich, plus a recently-played history panel,
/// mirrored top/bottom visualizer bars, a scrolling marquee and a
/// countdown-to-next-track readout.
pub fn render(frame: &mut Frame, app: &App) {
    let outer = Block::default()
        .title(" HUMANMUSIC.TV · Full ".bold().magenta())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(ratatui::style::Style::new().magenta());
    let inner = outer.inner(frame.area());
    frame.render_widget(outer, frame.area());

    let [main_area, history_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(28)]).areas(inner);

    render_main(frame, app, main_area);
    render_history(frame, app, history_area);
}

fn render_main(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let [
        logo,
        on_air,
        now_playing,
        top_bars,
        progress,
        volume,
        next_up,
        bottom_bars,
        marquee,
        error,
        footer,
    ] = Layout::vertical([
        Constraint::Length(HMTV_LOGO.len() as u16),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let logo_lines: Vec<Line> = HMTV_LOGO
        .iter()
        .map(|l| Line::from((*l).cyan().bold()))
        .collect();
    frame.render_widget(
        Paragraph::new(logo_lines).alignment(Alignment::Center),
        logo,
    );

    let is_paused = app.player_status.paused || app.now_playing.is_none();
    let blink_on = (app.tick_count / 5).is_multiple_of(2);
    let indicator = if is_paused {
        " ⏸ PAUSED ".black().on_yellow().bold()
    } else if blink_on {
        " ● ON AIR ".black().on_green().bold()
    } else {
        " ● ON AIR ".dim()
    };
    frame.render_widget(
        Paragraph::new(Line::from(indicator)).alignment(Alignment::Center),
        on_air,
    );

    if let Some(np) = &app.now_playing {
        let t = &np.data.current_track;
        let lines = vec![
            Line::from(t.title.clone().bold()).alignment(Alignment::Center),
            Line::from(vec![
                t.artist.clone().magenta(),
                format!(" · {}", t.year).dim(),
            ])
            .alignment(Alignment::Center),
        ];
        frame.render_widget(Paragraph::new(lines), now_playing);
    } else {
        frame.render_widget(
            Paragraph::new("connecting to HumanMusic.tv…".dim()).alignment(Alignment::Center),
            now_playing,
        );
    }

    // Mirrored bars: same levels rendered above and below the transport
    // controls for a symmetric "equalizer" look.
    let bars = bars_line(&app.visualizer_levels);
    frame.render_widget(
        Paragraph::new(bars.clone().cyan()).alignment(Alignment::Center),
        top_bars,
    );

    if app.now_playing.is_some() {
        let ratio = app.progress_ratio();
        let label = format!(
            "{} / {}",
            format_mmss(app.elapsed_secs()),
            format_mmss(app.duration_secs())
        );
        let gauge = Gauge::default()
            .gauge_style(ratatui::style::Style::new().magenta().on_dark_gray())
            .ratio(ratio)
            .label(label);
        frame.render_widget(gauge, progress);
    }

    let vol_ratio = (app.player_status.volume / 100.0).clamp(0.0, 1.0);
    let vol_gauge = Gauge::default()
        .gauge_style(ratatui::style::Style::new().cyan())
        .ratio(vol_ratio)
        .label(format!("vol {:.0}%", app.player_status.volume));
    frame.render_widget(vol_gauge, volume);

    if let Some(np) = &app.now_playing {
        let n = &np.data.next_track;
        let countdown = countdown_label(app.duration_secs() - app.elapsed_secs());
        let lines = vec![
            Line::from(vec!["next up ".dim(), countdown.yellow()]).alignment(Alignment::Center),
            Line::from(vec![
                n.artist.clone().bold(),
                " — ".dim(),
                n.title.clone().into(),
            ])
            .alignment(Alignment::Center),
        ];
        frame.render_widget(Paragraph::new(lines), next_up);
    }

    frame.render_widget(
        Paragraph::new(bars.cyan()).alignment(Alignment::Center),
        bottom_bars,
    );

    if let Some(np) = &app.now_playing {
        let t = &np.data.current_track;
        let text = format!("{}  —  {}   ***   ", t.artist, t.title);
        let scrolled = marquee_window(&text, marquee.width as usize, app.tick_count);
        frame.render_widget(Paragraph::new(scrolled.magenta()), marquee);
    }

    if let Some(msg) = &app.last_error {
        frame.render_widget(
            Paragraph::new(error_banner(msg)).alignment(Alignment::Center),
            error,
        );
    }

    frame.render_widget(
        Paragraph::new(footer_keybinds()).alignment(Alignment::Center),
        footer,
    );
}

fn render_history(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let block = Block::default()
        .title(" recently played ".bold())
        .borders(Borders::LEFT)
        .border_style(ratatui::style::Style::new().dim());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [antenna, list_area] = Layout::vertical([
        Constraint::Length(ANTENNA.len() as u16),
        Constraint::Fill(1),
    ])
    .areas(inner);

    let antenna_lines: Vec<Line> = ANTENNA.iter().map(|l| Line::from((*l).yellow())).collect();
    frame.render_widget(
        Paragraph::new(antenna_lines).alignment(Alignment::Center),
        antenna,
    );

    let items: Vec<ListItem> = if app.history.is_empty() {
        vec![ListItem::new("  (nothing yet)".dim())]
    } else {
        app.history
            .iter()
            .map(|t| {
                ListItem::new(Line::from(vec![
                    "  ".into(),
                    t.artist.clone().magenta(),
                    " — ".dim(),
                    t.title.clone().into(),
                ]))
            })
            .collect()
    };
    frame.render_widget(List::new(items), list_area);
}

/// Formats the remaining seconds in the current track as `mm:ss`. Derived
/// from the already skew-immune `elapsed_secs`/`duration_secs` (based on
/// mpv's own position or the API's `elapsed` field) rather than comparing
/// absolute epoch timestamps against the local wall clock.
fn countdown_label(remaining_secs: f64) -> String {
    format!("in {}", format_mmss(remaining_secs.max(0.0)))
}

/// Produces a `width`-wide sliding window over `text` (repeated/padded),
/// advancing one character every few ticks to create a marquee scroll.
fn marquee_window(text: &str, width: usize, tick_count: u64) -> String {
    if width == 0 {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return " ".repeat(width);
    }
    let len = chars.len();
    let offset = ((tick_count / 3) as usize) % len;
    (0..width).map(|i| chars[(offset + i) % len]).collect()
}

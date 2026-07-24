use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Gauge, Paragraph, Wrap};

use crate::app::App;
use crate::ui::ascii::{HMTV_LOGO, bars_line, format_mmss};
use crate::ui::{error_banner, header_rule, render_keybind_bar};

/// Screen 3: Rich. The ASCII wordmark banner, a blinking ON AIR indicator and
/// a volume-driven dynamic histogram sit above the transport controls.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = app.theme;
    let area = frame.area();

    let [
        header,
        spacer1,
        logo,
        on_air,
        spacer2,
        now_playing,
        artist_line,
        spacer3,
        visualizer,
        spacer4,
        progress,
        spacer5,
        volume,
        spacer6,
        next_label,
        next_line,
        filler,
        error,
        keybinds,
    ] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(HMTV_LOGO.len() as u16 + 1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(4),
    ])
    .areas(area);
    let _ = (spacer1, spacer2, spacer3, spacer4, spacer5, spacer6, filler);

    frame.render_widget(header_rule(theme, "HUMANMUSIC.TV · RICH"), header);

    let logo_lines: Vec<Line> = HMTV_LOGO
        .iter()
        .map(|l| Line::from((*l).fg(theme.accent()).bold()))
        .collect();
    frame.render_widget(
        Paragraph::new(logo_lines).alignment(Alignment::Center),
        logo,
    );

    let is_paused = app.player_status.paused || app.now_playing.is_none();
    let blink_on = (app.tick_count / 5).is_multiple_of(2);

    let indicator = if app.last_error.is_some() {
        " PLAYBACK ERROR ".bold().fg(theme.accent())
    } else if is_paused {
        " ⏸ PAUSED ".fg(theme.dim())
    } else if blink_on {
        " ● ON AIR ".bold().fg(theme.accent())
    } else {
        " ● ON AIR ".fg(theme.dim())
    };
    frame.render_widget(
        Paragraph::new(Line::from(indicator)).alignment(Alignment::Center),
        on_air,
    );

    if let Some(err) = &app.last_error {
        frame.render_widget(
            Paragraph::new(err.clone().fg(theme.accent()))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            now_playing,
        );
    } else if let Some(np) = &app.now_playing {
        let t = &np.data.current_track;
        frame.render_widget(
            Paragraph::new(t.title.clone().bold().fg(theme.fg())).alignment(Alignment::Center),
            now_playing,
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                t.artist.clone().fg(theme.accent()),
                format!(" · {}", t.year).fg(theme.dim()),
            ]))
            .alignment(Alignment::Center),
            artist_line,
        );
    } else {
        frame.render_widget(
            Paragraph::new("connecting to HumanMusic.tv…".fg(theme.dim()))
                .alignment(Alignment::Center),
            now_playing,
        );
    }

    let bars = bars_line(&app.visualizer_levels);
    frame.render_widget(
        Paragraph::new(bars.fg(theme.accent())).alignment(Alignment::Center),
        visualizer,
    );

    if app.now_playing.is_some() {
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
        frame.render_widget(gauge, progress);
    }

    let vol_val = app.player_status.volume.round() as usize;
    let width = 20;
    let filled = (vol_val * width) / 100;
    let bar: String = (0..width)
        .map(|i| if i < filled { '▓' } else { '░' })
        .collect();

    let vol_text = vec![
        Line::from(format!("Volume: {}%", vol_val)).alignment(Alignment::Center),
        Line::from(format!("│{}│", bar)).alignment(Alignment::Center),
    ];
    frame.render_widget(Paragraph::new(vol_text).fg(theme.dim()), volume);

    if let Some(np) = &app.now_playing {
        let n = &np.data.next_track;
        frame.render_widget(
            Paragraph::new("next up".fg(theme.dim())).alignment(Alignment::Center),
            next_label,
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                n.artist.clone().bold(),
                " — ".fg(theme.dim()),
                n.title.clone().fg(theme.fg()),
            ]))
            .alignment(Alignment::Center),
            next_line,
        );
    }

    if let Some(msg) = &app.last_error {
        frame.render_widget(
            Paragraph::new(error_banner(theme, msg)).alignment(Alignment::Center),
            error,
        );
    }

    render_keybind_bar(frame, keybinds, theme);
}

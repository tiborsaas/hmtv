use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph};

use crate::app::App;
use crate::ui::ascii::{HMTV_LOGO, bars_line, format_mmss};
use crate::ui::{error_banner, footer_keybinds};

/// Screen 3: Rich. Adds the ASCII wordmark banner, a blinking ON AIR
/// indicator and an animated (decorative, non-audio-reactive) visualizer.
pub fn render(frame: &mut Frame, app: &App) {
    let outer = Block::default()
        .title(" HUMANMUSIC.TV · Rich ".bold().magenta())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(ratatui::style::Style::new().magenta());
    let inner = outer.inner(frame.area());
    frame.render_widget(outer, frame.area());

    let [
        logo,
        on_air,
        now_playing,
        visualizer,
        progress,
        volume,
        next_up,
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
    ])
    .areas(inner);

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

    let bars = bars_line(&app.visualizer_levels);
    frame.render_widget(
        Paragraph::new(bars.cyan()).alignment(Alignment::Center),
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
        let lines = vec![
            Line::from("next up".dim()).alignment(Alignment::Center),
            Line::from(vec![
                n.artist.clone().bold(),
                " — ".dim(),
                n.title.clone().into(),
            ])
            .alignment(Alignment::Center),
        ];
        frame.render_widget(Paragraph::new(lines), next_up);
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

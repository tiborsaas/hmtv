use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Gauge, Paragraph};

use crate::app::App;
use crate::ui::ascii::format_mmss;
use crate::ui::{error_banner, header_rule, render_keybind_bar};

/// Screen 2: Standard. A themed header rule, a now-playing readout with real
/// progress/volume gauges, a "next up" preview, and a boxed keybind footer.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = app.theme;
    let area = frame.area();

    let [
        header,
        spacer1,
        now_playing,
        artist_line,
        spacer2,
        progress,
        spacer3,
        volume,
        spacer4,
        next_label,
        next_line,
        filler,
        error,
        keybinds,
    ] = Layout::vertical([
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
    let _ = (spacer1, spacer2, spacer3, spacer4, filler);

    frame.render_widget(header_rule(theme, "HUMANMUSIC.TV · STANDARD"), header);

    if let Some(np) = &app.now_playing {
        let t = &np.data.current_track;
        let status = if app.player_status.paused {
            " ⏸ PAUSED ".fg(theme.dim())
        } else {
            " ● ON AIR ".bold().fg(theme.accent())
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                status,
                format!("  {}", t.title).bold().fg(theme.fg()),
            ]))
            .alignment(Alignment::Center),
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

    let vol_ratio = (app.player_status.volume / 100.0).clamp(0.0, 1.0);
    let vol_gauge = Gauge::default()
        .gauge_style(Style::new().fg(theme.dim()))
        .ratio(vol_ratio)
        .label(format!("vol {:.0}%", app.player_status.volume));
    frame.render_widget(vol_gauge, volume);

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
            Paragraph::new(error_banner(msg)).alignment(Alignment::Center),
            error,
        );
    }

    render_keybind_bar(frame, keybinds, theme);
}

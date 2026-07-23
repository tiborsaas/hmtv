use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::ui::ascii::{format_mmss, thin_progress_bar};
use crate::ui::{error_banner, footer_keybinds};

/// Screen 1: Minimal. A single centered "now playing" line, a thin progress
/// bar, and a one-line footer. No borders, no chrome.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let [tag, main, progress, error, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new("HUMANMUSIC.TV".dim()).alignment(Alignment::Center),
        tag,
    );

    let line = if let Some(np) = &app.now_playing {
        let icon = if app.player_status.paused {
            "⏸ "
        } else {
            "▶ "
        };
        Line::from(vec![
            icon.cyan().bold(),
            np.data.current_track.artist.clone().bold(),
            " — ".dim(),
            np.data.current_track.title.clone().into(),
        ])
    } else {
        Line::from("connecting to HumanMusic.tv…".dim())
    };
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), main);

    if app.now_playing.is_some() {
        let width = (progress.width as usize).saturating_sub(20).clamp(10, 60);
        let bar = thin_progress_bar(width, app.progress_ratio());
        let label = format!(
            "{bar} {} / {}",
            format_mmss(app.elapsed_secs()),
            format_mmss(app.duration_secs())
        );
        frame.render_widget(
            Paragraph::new(label.dim()).alignment(Alignment::Center),
            progress,
        );
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

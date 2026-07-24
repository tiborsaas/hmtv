use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::app::App;

/// Screen 1: Minimal. Nothing but the title and artist, vertically centered.
/// No borders, no chrome, no controls hints — as bare as it gets.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = app.theme;
    let area = frame.area();
    let [_, content, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Fill(1),
    ])
    .areas(area);

    let lines = if let Some(np) = &app.now_playing {
        let t = &np.data.current_track;
        vec![
            Line::from(t.title.clone().bold().fg(theme.accent())).alignment(Alignment::Center),
            Line::from(t.artist.clone().fg(theme.dim())).alignment(Alignment::Center),
        ]
    } else {
        vec![
            Line::from("").alignment(Alignment::Center),
            Line::from("connecting…".fg(theme.dim())).alignment(Alignment::Center),
        ]
    };

    frame.render_widget(Paragraph::new(lines), content);
}

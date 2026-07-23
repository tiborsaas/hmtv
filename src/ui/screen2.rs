use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph};

use crate::app::App;
use crate::ui::ascii::format_mmss;
use crate::ui::{error_banner, footer_keybinds};

/// Screen 2: Standard. A bordered now-playing panel with a real progress
/// gauge, a volume indicator and a "next up" preview.
pub fn render(frame: &mut Frame, app: &App) {
    let outer = Block::default()
        .title(" HUMANMUSIC.TV · Standard ".bold().cyan())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(ratatui::style::Style::new().cyan());
    let inner = outer.inner(frame.area());
    frame.render_widget(outer, frame.area());

    let [now_playing, progress, volume, next_up, error, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    if let Some(np) = &app.now_playing {
        let t = &np.data.current_track;
        let status = if app.player_status.paused {
            "PAUSED".yellow().bold()
        } else {
            "ON AIR".green().bold()
        };
        let lines = vec![
            Line::from(vec![" ".into(), status, format!("  {}", t.title).bold()]),
            Line::from(vec![
                "   ".into(),
                t.artist.clone().magenta(),
                format!(" · {}", t.year).dim(),
            ]),
        ];
        frame.render_widget(Paragraph::new(lines), now_playing);
    } else {
        frame.render_widget(
            Paragraph::new(" connecting to HumanMusic.tv…".dim()),
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
            .gauge_style(ratatui::style::Style::new().cyan().on_dark_gray())
            .ratio(ratio)
            .label(label);
        frame.render_widget(gauge, progress);
    }

    let vol_ratio = (app.player_status.volume / 100.0).clamp(0.0, 1.0);
    let vol_gauge = Gauge::default()
        .gauge_style(ratatui::style::Style::new().magenta())
        .ratio(vol_ratio)
        .label(format!("vol {:.0}%", app.player_status.volume));
    frame.render_widget(vol_gauge, volume);

    if let Some(np) = &app.now_playing {
        let n = &np.data.next_track;
        let lines = vec![
            Line::from("next up".dim()),
            Line::from(vec![
                "  ".into(),
                n.artist.clone().bold(),
                " — ".dim(),
                n.title.clone().into(),
            ]),
        ];
        frame.render_widget(Paragraph::new(lines), next_up);
    }

    if let Some(msg) = &app.last_error {
        frame.render_widget(Paragraph::new(error_banner(msg)), error);
    }

    frame.render_widget(
        Paragraph::new(footer_keybinds()).alignment(Alignment::Center),
        footer,
    );
}

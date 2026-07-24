use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;

use crate::app::App;
use crate::ui::ascii::format_mmss;
use crate::ui::generative::abstract_barcode;
use crate::ui::header_rule;
use crate::ui::theme::Theme;

/// Screen 4: Full. A single abstract, barcode-like field that fills the view,
/// with a centered, padded text panel for the current track details.
pub fn render(frame: &mut Frame, app: &App) {
    let theme = app.theme;
    let area = frame.area();

    let [header, content] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    frame.render_widget(header_rule(theme, "HUMANMUSIC.TV · FULL"), header);
    abstract_barcode(frame.buffer_mut(), content, app.tick_count, theme);
    render_center_panel(frame.buffer_mut(), content, app, theme);
}

fn render_center_panel(buf: &mut Buffer, area: Rect, app: &App, theme: Theme) {
    let pad = 4u16;
    let title = app
        .now_playing
        .as_ref()
        .map(|np| np.data.current_track.title.clone())
        .unwrap_or_else(|| "connecting…".to_string());
    let artist = app
        .now_playing
        .as_ref()
        .map(|np| np.data.current_track.artist.clone())
        .unwrap_or_else(|| "HumanMusic.tv".to_string());
    let year = app
        .now_playing
        .as_ref()
        .map(|np| np.data.current_track.year.to_string())
        .unwrap_or_else(|| "????".to_string());

    let title_text = fit_text(&title, area.width as usize);
    let artist_text = fit_text(&artist, area.width as usize);
    let year_lines = ascii_art_year(&year);
    let message_text = next_message(app, theme);

    let content_width = title_text
        .chars()
        .count()
        .max(artist_text.chars().count())
        .max(message_text.chars().count())
        .max(
            year_lines
                .iter()
                .map(|l| l.chars().count())
                .max()
                .unwrap_or(0),
        );
    let content_height = 1 + 1 + year_lines.len() + 1;
    let panel_width = (content_width as u16)
        .saturating_add(pad * 2)
        .min(area.width);
    let panel_height = (content_height as u16)
        .saturating_add(pad * 2)
        .min(area.height);
    let panel_x = area.x + (area.width.saturating_sub(panel_width)) / 2;
    let panel_y = area.y + (area.height.saturating_sub(panel_height)) / 2;
    let panel = Rect {
        x: panel_x,
        y: panel_y,
        width: panel_width,
        height: panel_height,
    };

    let bg = Style::new().bg(theme.dim());
    for y in panel.top()..panel.bottom() {
        for x in panel.left()..panel.right() {
            buf.set_string(x, y, " ", bg);
        }
    }

    let inner = Rect {
        x: panel.x + pad,
        y: panel.y + pad,
        width: panel.width.saturating_sub(pad * 2),
        height: panel.height.saturating_sub(pad * 2),
    };

    let title_style = Style::new().fg(theme.accent()).bg(theme.dim());
    let artist_style = Style::new().fg(theme.fg()).bg(theme.dim());
    let year_style = Style::new().fg(theme.accent()).bg(theme.dim());

    draw_centered_line(buf, inner, 0, &title_text, title_style);
    draw_centered_line(buf, inner, 1, &artist_text, artist_style);

    let year_y = inner.y + 2;
    for (row_idx, line) in year_lines.iter().enumerate() {
        let year_x = inner.x + (inner.width.saturating_sub(line.chars().count() as u16)) / 2;
        let y = year_y + row_idx as u16;
        if y < inner.bottom() {
            buf.set_string(year_x, y, line, year_style);
        }
    }

    let message_y = inner.y + 2 + year_lines.len() as u16;
    if message_y < inner.bottom() {
        draw_centered_line(buf, inner, message_y - inner.y, &message_text, artist_style);
    }
}

fn draw_centered_line(buf: &mut Buffer, area: Rect, row: u16, text: &str, style: Style) {
    let width = text.chars().count().min(area.width as usize);
    let x = area.x + (area.width.saturating_sub(width as u16)) / 2;
    let y = area.y + row;
    if y < area.bottom() {
        buf.set_string(x, y, text, style);
    }
}

fn fit_text(text: &str, width: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        return text.to_string();
    }
    chars.into_iter().take(width).collect()
}

fn ascii_art_year(year: &str) -> Vec<String> {
    let patterns = [
        [" _ ", "| |", "|_|"],
        ["   ", "  |", "  |"],
        [" _ ", " _|", "|_ "],
        [" _ ", " _|", " _|"],
        ["   ", "|_|", "  |"],
        [" _ ", "|_ ", " _|"],
        [" _ ", "|_ ", "|_|"],
        [" _ ", "  |", "  |"],
        [" _ ", "|_|", "|_|"],
        [" _ ", "|_|", " _|"],
        [" ? ", " ? ", " ? "],
    ];

    let digits: Vec<char> = year.chars().collect();
    if digits.is_empty() {
        return vec!["????".to_string()];
    }

    let mut lines = vec![String::new(); 3];
    for digit in digits {
        let d = digit.to_digit(10).unwrap_or(10) as usize;
        let pattern = patterns.get(d).unwrap_or(patterns.last().unwrap());
        for (idx, row) in pattern.iter().enumerate() {
            if idx < lines.len() {
                lines[idx].push(' ');
                lines[idx].push_str(row);
            }
        }
    }

    lines
}

fn next_message(app: &App, _theme: Theme) -> String {
    if let Some(np) = &app.now_playing {
        let remaining = app.duration_secs() - app.elapsed_secs();
        if remaining <= 30.0 && !np.data.next_track.title.is_empty() {
            format!("Next: {}", np.data.next_track.title)
        } else {
            format!("in {}", format_mmss(remaining.max(0.0)))
        }
    } else {
        "waiting for stream…".to_string()
    }
}

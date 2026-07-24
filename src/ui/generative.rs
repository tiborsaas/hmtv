//! Procedurally generated decorative art for the "Full" screen. Everything
//! here is computed per-frame from simple geometry (no external assets),
//! scales to whatever panel size it is given, and animates gently using the
//! app's tick counter. This is deliberately generative/demoscene-style rather
//! than a fixed ASCII-art block, so it holds up at any terminal size.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::ui::theme::Theme;

/// A dense, abstract barcode-like field that slowly scrolls and evolves.
/// It is meant to feel more like a generative visual artifact than a literal
/// title/transport readout.
pub fn abstract_barcode(buf: &mut Buffer, area: Rect, tick: u64, theme: Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let dim_style = Style::new().fg(theme.dim());
    let accent_style = Style::new().fg(theme.accent());
    let pattern = "011010011001011010110010101001";
    let stride = pattern.len();
    let scroll = ((tick / 2) as usize) % stride;

    for y in area.top()..area.bottom() {
        let row_shift = ((y as usize + (tick as usize / 4)) % 5) as i64;
        for x in area.left()..area.right() {
            let base = (x as i64 + y as i64 * 3 + row_shift) % 11;
            let stream_idx = (x as usize + scroll + y as usize * 2) % stride;
            let bit = pattern.as_bytes()[stream_idx] as char;
            let glyph = if bit == '1' { "█" } else { "░" };
            let is_stream = ((x as i64 + y as i64 + (tick as i64 / 4)) % 7) == 0;

            if is_stream {
                buf.set_string(x, y, glyph, accent_style);
            } else if base < 3 {
                buf.set_string(x, y, "▌", accent_style);
            } else if base < 6 {
                buf.set_string(x, y, "│", dim_style);
            } else if base < 8 {
                buf.set_string(x, y, "╱", dim_style);
            } else {
                buf.set_string(x, y, " ", dim_style);
            }
        }
    }
}

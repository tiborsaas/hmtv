//! Original decorative ASCII art and small rendering helpers used across the
//! four screens. Nothing here is copied from any external source.

/// Large block-letter "HMTV" wordmark used on the Rich/Full screens.
pub const HMTV_LOGO: [&str; 5] = [
    "█   █  █   █  █████  █   █",
    "█   █  ██ ██    █    █   █",
    "█████  █ █ █    █    █   █",
    "█   █  █   █    █     █ █ ",
    "█   █  █   █    █      █  ",
];

/// Small antenna/broadcast-tower glyph for decorative headers.
pub const ANTENNA: [&str; 4] = ["  \\|/  ", " --o-- ", "   |   ", "  /|\\  "];

/// One of 8 Unicode block-height glyphs, `level` in `0..=8` (0 = empty).
pub fn bar_glyph(level: u8) -> char {
    const GLYPHS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    GLYPHS[level.min(8) as usize]
}

/// Renders a row of bar-height glyphs from a slice of levels (each `0..=8`).
pub fn bars_line(levels: &[u8]) -> String {
    levels.iter().map(|&l| bar_glyph(l)).collect()
}

/// Formats a duration given in (fractional) seconds as `mm:ss`.
pub fn format_mmss(total_secs: f64) -> String {
    let total_secs = total_secs.max(0.0).round() as i64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{mins:02}:{secs:02}")
}

/// A thin single-line progress indicator, e.g. `▐████████░░░░░▌`.
pub fn thin_progress_bar(width: usize, ratio: f64) -> String {
    let ratio = ratio.clamp(0.0, 1.0);
    let filled = ((width as f64) * ratio).round() as usize;
    let filled = filled.min(width);
    let mut s = String::with_capacity(width + 2);
    s.push('▐');
    for i in 0..width {
        s.push(if i < filled { '█' } else { '░' });
    }
    s.push('▌');
    s
}

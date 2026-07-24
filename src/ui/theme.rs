//! Color theme system. Each theme defines an accent color, a dimmer shade of
//! the same hue for chrome/secondary text, and a soft foreground color for
//! body text, keeping the overall look a monochrome-leaning terminal palette
//! with a single accent hue rather than a full multi-color UI.

use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Green,
    Monochrome,
    Amber,
    Red,
    Blue1,
    Blue2,
}

impl Theme {
    pub const ALL: [Theme; 6] = [
        Theme::Monochrome,
        Theme::Green,
        Theme::Amber,
        Theme::Red,
        Theme::Blue1,
        Theme::Blue2,
    ];

    /// Cycles to the next theme in `ALL`, wrapping around.
    pub fn next(self) -> Theme {
        let idx = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            Theme::Monochrome => "MONOCHROME",
            Theme::Green => "GREEN",
            Theme::Amber => "AMBER",
            Theme::Red => "RED",
            Theme::Blue1 => "BLUE I",
            Theme::Blue2 => "BLUE II",
        }
    }

    pub fn dark(self) -> Color {
        Color::Rgb(10, 10, 10)
    }

    /// Bright accent color: headings, highlights, active bars.
    pub fn accent(self) -> Color {
        match self {
            Theme::Monochrome => Color::Rgb(230, 230, 230),
            Theme::Green => Color::Rgb(90, 240, 140),
            Theme::Amber => Color::Rgb(255, 176, 40),
            Theme::Red => Color::Rgb(255, 100, 100),
            Theme::Blue1 => Color::Rgb(90, 180, 255),
            Theme::Blue2 => Color::Rgb(140, 130, 255),
        }
    }

    /// Dim/secondary shade of the same hue: chrome, borders, muted text.
    pub fn dim(self) -> Color {
        match self {
            Theme::Monochrome => Color::Rgb(120, 120, 120),
            Theme::Green => Color::Rgb(45, 115, 75),
            Theme::Amber => Color::Rgb(140, 100, 30),
            Theme::Red => Color::Rgb(140, 55, 55),
            Theme::Blue1 => Color::Rgb(50, 90, 140),
            Theme::Blue2 => Color::Rgb(70, 65, 140),
        }
    }

    /// Soft body-text foreground, slightly muted so the accent still pops.
    pub fn fg(self) -> Color {
        match self {
            Theme::Monochrome => Color::Rgb(215, 215, 215),
            _ => Color::Rgb(220, 218, 208),
        }
    }
}

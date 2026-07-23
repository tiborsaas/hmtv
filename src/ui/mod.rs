pub mod ascii;
mod screen1;
mod screen2;
mod screen3;
mod screen4;

use ratatui::Frame;
use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        1 => screen1::render(frame, app),
        2 => screen2::render(frame, app),
        3 => screen3::render(frame, app),
        4 => screen4::render(frame, app),
        _ => screen2::render(frame, app),
    }
}

/// Shared footer keybind hint line, reused by every screen.
pub fn footer_keybinds() -> Line<'static> {
    Line::from(vec![
        " 1-4 ".bold().cyan(),
        "screens ".dim(),
        " space ".bold().cyan(),
        "pause ".dim(),
        " +/- ".bold().cyan(),
        "volume ".dim(),
        " r ".bold().cyan(),
        "resync ".dim(),
        " q ".bold().cyan(),
        "quit ".dim(),
    ])
}

/// Renders the `Action::Error` banner, if any, as a single dismissible-looking
/// line. Screens call this last so it overlays near the bottom.
pub fn error_banner(msg: &str) -> Line<'static> {
    Line::from(vec![
        " ERROR ".black().on_red().bold(),
        format!(" {msg}").red(),
    ])
}

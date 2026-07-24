use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::action::Action;

/// Maps a terminal input event to an `Action`, if it corresponds to a known
/// key binding. Returns `None` for events we don't care about (e.g. key
/// release events, resize, mouse).
pub fn handle_event(event: Event) -> Option<Action> {
    match event {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return None;
            }
            match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Esc => Some(Action::ClearError),
                KeyCode::Char('1') => Some(Action::SwitchScreen(1)),
                KeyCode::Char('2') => Some(Action::SwitchScreen(2)),
                KeyCode::Char('3') => Some(Action::SwitchScreen(3)),
                KeyCode::Char('4') => Some(Action::SwitchScreen(4)),
                KeyCode::Char(' ') | KeyCode::Char('p') => Some(Action::TogglePause),
                KeyCode::Char('+') | KeyCode::Char('=') => Some(Action::VolumeUp),
                KeyCode::Char('-') | KeyCode::Char('_') => Some(Action::VolumeDown),
                KeyCode::Char('r') => Some(Action::Resync),
                KeyCode::Char('t') => Some(Action::CycleTheme),
                _ => None,
            }
        }
        _ => None,
    }
}

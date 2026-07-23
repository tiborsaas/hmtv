use crate::api::NowPlayingUpdate;
use crate::player::PlayerStatus;

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Quit,
    SwitchScreen(u8),
    TogglePause,
    VolumeUp,
    VolumeDown,
    Resync,
    TracksUpdated(Box<NowPlayingUpdate>),
    PlayerStatusChanged(PlayerStatus),
    Error(String),
    ClearError,
}

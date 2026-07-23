use std::collections::VecDeque;
use std::sync::mpsc::Sender as StdSender;

use crate::action::Action;
use crate::api::{NowPlayingUpdate, TrackInfo};
use crate::player::{PlayerCommand, PlayerStatus};

const MAX_HISTORY: usize = 8;
const VISUALIZER_COLUMNS: usize = 48;

pub struct App {
    pub screen: u8,
    pub should_quit: bool,
    pub now_playing: Option<NowPlayingUpdate>,
    pub player_status: PlayerStatus,
    pub history: VecDeque<TrackInfo>,
    pub last_error: Option<String>,
    pub tick_count: u64,
    /// Decorative, non-audio-reactive visualizer bar heights (0..=8 each).
    pub visualizer_levels: Vec<u8>,
    player_cmd_tx: StdSender<PlayerCommand>,
}

impl App {
    pub fn new(player_cmd_tx: StdSender<PlayerCommand>) -> Self {
        Self {
            screen: 2,
            should_quit: false,
            now_playing: None,
            player_status: PlayerStatus::default(),
            history: VecDeque::with_capacity(MAX_HISTORY),
            last_error: None,
            tick_count: 0,
            visualizer_levels: vec![1; VISUALIZER_COLUMNS],
            player_cmd_tx,
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Tick => {
                self.tick_count = self.tick_count.wrapping_add(1);
                self.step_visualizer();
            }
            Action::Quit => self.should_quit = true,
            Action::SwitchScreen(n) => {
                if (1..=4).contains(&n) {
                    self.screen = n;
                }
            }
            Action::TogglePause => {
                let _ = self.player_cmd_tx.send(PlayerCommand::TogglePause);
            }
            Action::VolumeUp => {
                let _ = self.player_cmd_tx.send(PlayerCommand::VolumeUp);
            }
            Action::VolumeDown => {
                let _ = self.player_cmd_tx.send(PlayerCommand::VolumeDown);
            }
            Action::Resync => {
                if let Some(np) = &self.now_playing {
                    let resume_secs = np.estimated_elapsed_secs();
                    let _ = self
                        .player_cmd_tx
                        .send(PlayerCommand::Resync { resume_secs });
                }
            }
            Action::TracksUpdated(update) => self.handle_tracks_updated(update),
            Action::PlayerStatusChanged(status) => self.player_status = status,
            Action::Error(msg) => self.last_error = Some(msg),
            Action::ClearError => self.last_error = None,
        }
    }

    fn handle_tracks_updated(&mut self, update: Box<NowPlayingUpdate>) {
        let is_new_track = self
            .now_playing
            .as_ref()
            .map(|np| np.data.current_track.id != update.data.current_track.id)
            .unwrap_or(true);

        if is_new_track {
            if let Some(prev) = &self.now_playing {
                self.history.push_front(prev.data.current_track.clone());
                self.history.truncate(MAX_HISTORY);
            }
            let resume_secs = update.estimated_elapsed_secs();
            let _ = self.player_cmd_tx.send(PlayerCommand::Load {
                ytid: update.data.current_track.ytid.clone(),
                resume_secs,
            });
        }

        self.now_playing = Some(*update);
    }

    /// Decorative random-walk animation for the visualizer bars (screens 3/4).
    /// Not derived from real audio data.
    fn step_visualizer(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let is_paused = self.player_status.paused || self.now_playing.is_none();
        for level in self.visualizer_levels.iter_mut() {
            if is_paused {
                *level = level.saturating_sub(1);
                continue;
            }
            let delta: i8 = rng.gen_range(-2..=2);
            let new_val = (*level as i8 + delta).clamp(0, 8);
            *level = new_val as u8;
        }
    }

    pub fn duration_secs(&self) -> f64 {
        self.now_playing
            .as_ref()
            .map(|np| np.data.current_track.duration as f64)
            .unwrap_or(0.0)
    }

    /// Best-known elapsed playback position: prefers mpv's own reported
    /// position (ground truth) once it has loaded the current track, and
    /// falls back to an API-time-based estimate otherwise (e.g. right after
    /// a track switch, before mpv has started reporting `time-pos`).
    pub fn elapsed_secs(&self) -> f64 {
        let Some(np) = &self.now_playing else {
            return 0.0;
        };
        let mpv_has_track = self.player_status.connected
            && self.player_status.loaded_ytid.as_deref()
                == Some(np.data.current_track.ytid.as_str());
        if mpv_has_track && self.player_status.position_secs > 0.0 {
            self.player_status.position_secs
        } else {
            np.estimated_elapsed_secs()
        }
    }

    pub fn progress_ratio(&self) -> f64 {
        let dur = self.duration_secs();
        if dur <= 0.0 {
            0.0
        } else {
            (self.elapsed_secs() / dur).clamp(0.0, 1.0)
        }
    }
}

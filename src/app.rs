use std::collections::VecDeque;
use std::sync::mpsc::Sender as StdSender;

use crate::action::Action;
use crate::api::{NowPlayingUpdate, TrackInfo};
use crate::player::{PlayerCommand, PlayerStatus};
use crate::ui::theme::Theme;

const MAX_HISTORY: usize = 8;
const VISUALIZER_COLUMNS: usize = 48;

pub struct App {
    pub screen: u8,
    pub theme: Theme,
    pub should_quit: bool,
    pub now_playing: Option<NowPlayingUpdate>,
    pub player_status: PlayerStatus,
    pub history: VecDeque<TrackInfo>,
    pub last_error: Option<String>,
    pub tick_count: u64,
    /// Histogram-style visualizer bar heights (0..=8 each). The baseline
    /// height tracks the current volume level, with per-tick jitter for a
    /// lively look; it is not derived from real audio spectrum data.
    pub visualizer_levels: Vec<u8>,
    player_cmd_tx: StdSender<PlayerCommand>,
}

impl App {
    pub fn new(player_cmd_tx: StdSender<PlayerCommand>) -> Self {
        Self {
            screen: 3,
            theme: Theme::default(),
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
            Action::OpenVideo => {
                if let Some(np) = &self.now_playing {
                    let url = format!("https://www.youtube.com/watch?v={}", np.data.current_track.ytid);
                    let _ = std::process::Command::new("open")
                        .arg(url)
                        .spawn();
                }
            }
            Action::CycleTheme => self.theme = self.theme.next(),
            Action::TracksUpdated(update) => self.handle_tracks_updated(update),
            Action::PlayerStatusChanged(status) => {
                // Only update last_error from player status if it's an error.
                // We avoid clearing errors from other sources (like API poller) here.
                if status.error.is_some() {
                    self.last_error = status.error.clone();
                } else if let Some(err) = &self.last_error {
                    // Heuristic: clear it only if it looks like a player error we previously set.
                    if err.contains("stopped unexpectedly") || err.contains("load failed") || err.contains("MPV") {
                        self.last_error = None;
                    }
                }
                self.player_status = status;
            }
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

    /// Histogram-style visualizer animation (screens 3/4). The baseline bar
    /// height tracks the current volume (louder = taller), with per-tick
    /// jitter smoothed toward that baseline for a lively but not-too-jumpy
    /// look. This is decorative and not derived from real audio data, since
    /// mpv's IPC does not expose spectrum/FFT data.
    fn step_visualizer(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let is_paused = self.player_status.paused || self.now_playing.is_none();
        let baseline = ((self.player_status.volume / 100.0).clamp(0.0, 1.0) * 8.0).round() as i8;
        for level in self.visualizer_levels.iter_mut() {
            if is_paused {
                *level = level.saturating_sub(1);
                continue;
            }
            let jitter: i8 = rng.gen_range(-3..=3);
            let target = (baseline + jitter).clamp(0, 8);
            let cur = *level as i8;
            let next = cur + (target - cur).signum();
            *level = next.clamp(0, 8) as u8;
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

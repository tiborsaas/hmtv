use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;

pub const API_URL: &str = "https://humanmusic.tv/api/v1.0/tracks";

// Some fields (original_duration, cue_in, cut_from_end) mirror the external
// API schema but aren't used by the UI yet; keep them for completeness.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct TrackInfo {
    pub start: i64,
    pub id: String,
    pub ytid: String,
    pub title: String,
    pub artist: String,
    pub year: i32,
    #[serde(rename = "originalDuration")]
    pub original_duration: i64,
    pub duration: i64,
    #[serde(rename = "cueIn")]
    pub cue_in: i64,
    #[serde(rename = "cutFromEnd")]
    pub cut_from_end: i64,
    /// Only present on `currentTrack` in the API response.
    #[serde(default)]
    pub elapsed: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NowPlaying {
    #[serde(rename = "currentTrack")]
    pub current_track: TrackInfo,
    #[serde(rename = "nextTrack")]
    pub next_track: TrackInfo,
}

/// A `NowPlaying` snapshot paired with the local monotonic instant it was
/// received at, so consumers can compensate for time elapsed since the fetch
/// when computing the correct playback resume position.
#[derive(Debug, Clone)]
pub struct NowPlayingUpdate {
    pub data: NowPlaying,
    pub fetched_at: Instant,
}

impl NowPlayingUpdate {
    /// Best estimate of the current track's elapsed playback position right
    /// now, combining the API-reported `elapsed` with the time that has
    /// passed locally since the response was received.
    pub fn estimated_elapsed_secs(&self) -> f64 {
        self.data.current_track.elapsed.unwrap_or(0.0) + self.fetched_at.elapsed().as_secs_f64()
    }
}

pub async fn fetch_now_playing(client: &reqwest::Client) -> Result<NowPlaying, String> {
    let resp = client
        .get(API_URL)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    let resp = resp
        .error_for_status()
        .map_err(|e| format!("bad response: {e}"))?;
    resp.json::<NowPlaying>()
        .await
        .map_err(|e| format!("invalid response body: {e}"))
}

/// Background task that periodically polls the HumanMusic.tv API and sends
/// `Action::TracksUpdated` on every successful fetch.
///
/// The poll interval adapts: it shortens as the scheduled `nextTrack.start`
/// time approaches so the track switch is detected close to on-time, without
/// needing a second dedicated timer task racing against this one.
pub async fn run_api_poller(action_tx: UnboundedSender<Action>) {
    const SAFETY_POLL_SECS: i64 = 15;

    let client = match reqwest::Client::builder()
        .user_agent("hmtv/0.1 (terminal radio player)")
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = action_tx.send(Action::Error(format!("failed to build HTTP client: {e}")));
            return;
        }
    };

    loop {
        match fetch_now_playing(&client).await {
            Ok(data) => {
                let wait_secs = next_wait_secs(&data, SAFETY_POLL_SECS);
                let update = NowPlayingUpdate {
                    data,
                    fetched_at: Instant::now(),
                };
                if action_tx
                    .send(Action::TracksUpdated(Box::new(update)))
                    .is_err()
                {
                    return;
                }
                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            }
            Err(e) => {
                if action_tx
                    .send(Action::Error(format!("HumanMusic.tv API: {e}")))
                    .is_err()
                {
                    return;
                }
                tokio::time::sleep(Duration::from_secs(SAFETY_POLL_SECS as u64)).await;
            }
        }
    }
}

/// Seconds to wait before the next poll. Deliberately avoids comparing
/// absolute epoch timestamps (`start`) against the local wall clock, since
/// that would be sensitive to any clock skew between this machine and the
/// HumanMusic.tv server. Instead it derives the remaining time in the
/// current track purely from the server-reported `duration`/`elapsed`
/// fields, which are self-consistent within a single response.
fn next_wait_secs(data: &NowPlaying, safety_poll_secs: i64) -> u64 {
    let elapsed = data.current_track.elapsed.unwrap_or(0.0);
    let remaining = (data.current_track.duration as f64 - elapsed).max(0.0) as i64;
    let wait = if remaining > 0 && remaining < safety_poll_secs {
        remaining + 1
    } else {
        safety_poll_secs
    };
    wait.max(1) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(elapsed: f64, duration: i64) -> TrackInfo {
        TrackInfo {
            start: 0,
            id: "id".into(),
            ytid: "yt".into(),
            title: "t".into(),
            artist: "a".into(),
            year: 2020,
            original_duration: duration,
            duration,
            cue_in: 0,
            cut_from_end: 0,
            elapsed: Some(elapsed),
        }
    }

    #[test]
    fn falls_back_to_safety_poll_when_far_away() {
        let np = NowPlaying {
            current_track: track(10.0, 1000),
            next_track: track(0.0, 200),
        };
        assert_eq!(next_wait_secs(&np, 15), 15);
    }

    #[test]
    fn shortens_wait_near_switch() {
        let np = NowPlaying {
            current_track: track(100.0, 105),
            next_track: track(0.0, 179),
        };
        assert_eq!(next_wait_secs(&np, 15), 6);
    }
}

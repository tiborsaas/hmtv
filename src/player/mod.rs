use std::io::{BufRead, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use mpvipc::{Mpv, NumberChangeOptions, SeekOptions};
use tokio::sync::watch;

/// Commands sent from the app to the dedicated mpv-owning thread.
#[derive(Debug, Clone)]
pub enum PlayerCommand {
    /// Load a new YouTube video (audio only) and seek to `resume_secs`.
    Load {
        ytid: String,
        resume_secs: f64,
    },
    TogglePause,
    VolumeUp,
    VolumeDown,
    /// Force-seek the currently loaded track to `resume_secs`.
    Resync {
        resume_secs: f64,
    },
    Shutdown,
}

/// Latest known state of the mpv backend, published to the UI thread.
#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub connected: bool,
    pub paused: bool,
    pub position_secs: f64,
    pub volume: f64,
    pub loaded_ytid: Option<String>,
    pub error: Option<String>,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            connected: false,
            paused: false,
            position_secs: 0.0,
            volume: 80.0,
            loaded_ytid: None,
            error: None,
        }
    }
}

/// Spawns the OS thread that owns the mpv subprocess and its IPC connection.
/// Communication happens via a plain `mpsc` command channel (in) and a
/// `tokio::sync::watch` status channel (out), which can be driven from a
/// non-async thread.
pub fn spawn_player_thread(
    cmd_rx: Receiver<PlayerCommand>,
    status_tx: watch::Sender<PlayerStatus>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = run_player(cmd_rx, status_tx.clone()) {
            let mut status = status_tx.borrow().clone();
            status.connected = false;
            status.error = Some(e);
            let _ = status_tx.send(status);
        }
    })
}

fn socket_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("hmtv-mpv-{}.sock", std::process::id()))
}

fn run_player(
    cmd_rx: Receiver<PlayerCommand>,
    status_tx: watch::Sender<PlayerStatus>,
) -> Result<(), String> {
    let socket = socket_path();
    let _ = std::fs::remove_file(&socket);

    let mut child: Child = Command::new("mpv")
        .arg("--no-video")
        .arg("--idle=yes")
        .arg(format!("--input-ipc-server={}", socket.display()))
        .arg("--really-quiet")
        .arg("--volume=80")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null()) // TODO: maybe log to a file instead of null
        .spawn()
        .map_err(|e| format!("failed to spawn mpv: {e}"))?;

    // Wait for mpv to create the IPC socket file.
    let deadline = Instant::now() + Duration::from_secs(5);
    while !socket.exists() {
        if Instant::now() > deadline {
            let _ = child.kill();
            return Err("timed out waiting for mpv IPC socket".into());
        }
        if let Ok(Some(exit)) = child.try_wait() {
            return Err(format!("mpv exited early ({exit})"));
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let mpv = Mpv::connect(socket.to_str().ok_or("socket path is not valid UTF-8")?)
        .map_err(|e| format!("mpv IPC connect failed: {e:?}"))?;

    // Create a shadow connection just for listening to events.
    // This avoids mpvipc's behavior of swallowing events during synchronous commands.
    let event_stream =
        UnixStream::connect(&socket).map_err(|e| format!("event socket failed: {e}"))?;
    event_stream
        .set_nonblocking(true)
        .map_err(|e| format!("failed to set nonblocking: {e}"))?;
    let mut event_reader = BufReader::new(event_stream.try_clone().map_err(|e| format!("stream clone failed: {e}"))?);
    let mut event_buf = String::new();

    let mut status = PlayerStatus {
        connected: true,
        ..Default::default()
    };
    // Deferred resume position applied on the next `file-loaded` event,
    // adjusted by time spent loading/buffering after `loadfile`.
    let mut pending_seek: Option<(f64, Instant)> = None;
    let _ = status_tx.send(status.clone());

    loop {
        // Drain events from the shadow connection.
        loop {
            event_buf.clear();
            match event_reader.read_line(&mut event_buf) {
                Ok(0) => break, // Socket closed
                Ok(_) => {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&event_buf) {
                        if val["event"] == "end-file" && val["reason"] == "error" {
                            let err_msg = val["file_error"]
                                .as_str()
                                .or(val["error"].as_str())
                                .unwrap_or("stream failed");
                            status.error = Some(format!("MPV Error: {err_msg}"));
                        } else if val["event"] == "file-loaded" {
                            if let Some((base_secs, started_at)) = pending_seek.take() {
                                let target_secs = base_secs + started_at.elapsed().as_secs_f64();
                                if target_secs > 1.0 {
                                    let _ = mpv.seek(target_secs, SeekOptions::Absolute);
                                }
                            }
                            let _ = mpv.set_property("pause", false);
                            status.paused = false;
                            status.error = None;
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        // Drain any pending commands without blocking.
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                PlayerCommand::Load { ytid, resume_secs } => {
                    let url = format!("https://www.youtube.com/watch?v={ytid}");
                    if let Err(e) = mpv.run_command_raw("loadfile", &[&url, "replace"]) {
                        status.error = Some(format!("load failed: {e:?}"));
                    } else {
                        status.error = None;
                        status.loaded_ytid = Some(ytid);
                        pending_seek = Some((resume_secs, Instant::now()));
                    }
                }
                PlayerCommand::TogglePause => {
                    let _ = mpv.toggle();
                }
                PlayerCommand::VolumeUp => {
                    let _ = mpv.set_volume(5.0, NumberChangeOptions::Increase);
                }
                PlayerCommand::VolumeDown => {
                    let _ = mpv.set_volume(5.0, NumberChangeOptions::Decrease);
                }
                PlayerCommand::Resync { resume_secs } => {
                    let _ = mpv.seek(resume_secs, SeekOptions::Absolute);
                }
                PlayerCommand::Shutdown => {
                    // Drop (disconnect) the IPC connection while mpv is
                    // still alive. mpvipc's `Drop for Mpv` calls
                    // `stream.shutdown(..).expect(..)` internally, which
                    // panics if the socket is already closed - which is
                    // exactly what happens if we ask mpv to `quit` first
                    // and then let `mpv` drop afterwards. Killing the
                    // process directly avoids that race entirely.
                    drop(mpv);
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = std::fs::remove_file(&socket);
                    return Ok(());
                }
            }
        }

        if let Ok(pos) = mpv.get_property::<f64>("time-pos") {
            status.position_secs = pos;
        }
        if let Ok(paused) = mpv.get_property::<bool>("pause") {
            status.paused = paused;
        }
        if let Ok(vol) = mpv.get_property::<f64>("volume") {
            status.volume = vol;
        }

        let _ = status_tx.send(status.clone());

        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Checks that the external binaries hmtv depends on are available on PATH.
/// Returns a human-readable error message with install hints if not.
pub fn check_prerequisites() -> Result<(), String> {
    for bin in ["mpv", "yt-dlp"] {
        let ok = Command::new(bin)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            return Err(format!(
                "`{bin}` was not found on your PATH.\n\n\
                 hmtv needs mpv (audio playback) and yt-dlp (YouTube stream resolution).\n\
                 Install them, e.g.:\n\
                 \x20\x20macOS:   brew install mpv yt-dlp\n\
                 \x20\x20Debian:  sudo apt install mpv && pipx install yt-dlp"
            ));
        }
    }
    // Flush stdout in case caller prints progress before this returns.
    let _ = std::io::stdout().flush();
    Ok(())
}

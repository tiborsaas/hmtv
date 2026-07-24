# hmtv

Terminal radio player for HumanMusic.tv, built with Rust, ratatui, and mpv.

It polls the HumanMusic.tv API, plays YouTube audio through mpv (audio-only), and renders four switchable terminal screens from a bare-minimum title/artist line up to a demoscene-style generative art screen.

## Features

- Live track data from HumanMusic.tv API
- Audio playback via mpv JSON IPC backend
- Four TUI screens (keys 1-4), from a bare title/artist line to a demoscene-style generative art screen
- 7 monochrome-leaning color themes, cycled with `t` (Monochrome, Green, Amber, Red, Blue I, Blue II, Teal)
- Volume-driven dynamic histogram visualizer bars
- Play/pause, volume control, manual resync
- Graceful startup checks for required binaries

## Requirements

- Rust (Cargo)
- mpv
- yt-dlp

Why yt-dlp is needed: mpv uses it to resolve/play YouTube stream sources.

## Install Dependencies

### macOS (Homebrew)

```bash
brew install mpv yt-dlp
```

### Debian/Ubuntu

```bash
sudo apt update
sudo apt install -y mpv
pipx install yt-dlp
```

## Build and Run

From the project root:

```bash
cargo run
```

Release build:

```bash
cargo build --release
```

Release binary path:

```text
target/release/hmtv
```

## Development Commands

Format check:

```bash
cargo fmt --check
```

Lint:

```bash
cargo clippy --all-features -- -D warnings
```

Tests:

```bash
cargo test
```

## Keybindings

- 1: Minimal screen
- 2: Standard screen
- 3: Rich screen
- 4: Full screen
- Space or p: Toggle pause
- - or =: Volume up
- - or \_: Volume down
- r: Resync playback to API timeline
- t: Cycle color theme
- Esc: Clear visible error banner
- q: Quit

## Screens

1. Minimal: just the title and artist, centered, no chrome
2. Standard: themed header rule, progress + volume gauges, next-up preview, boxed keybind footer
3. Rich: ASCII logo, blinking on-air badge, volume-driven histogram bars
4. Full: demoscene-style layout — procedural generative art panels (breathing disc over barcode stripes, converging concentric-square tunnel), a volume VU rail, mirrored histogram bars, marquee, and countdown to the next track

## Themes

Press `t` to cycle through 7 color themes: Monochrome, Green, Amber, Red, Blue I, Blue II, Teal. Each theme drives an accent color, a dim/chrome color, and body text color used consistently across all four screens; the active theme name is shown in the footer's `T` key box.

## Data Source

Current/next track API endpoint:

```text
https://humanmusic.tv/api/v1.0/tracks
```

Example response shape:

```json
{
  "currentTrack": {
    "start": 1784838266,
    "id": "8HysWMFzSH89nDX8T9GHkK",
    "ytid": "NnWy5qgdUqE",
    "title": "Tealeaf dancers",
    "artist": "Flying Lotus ",
    "year": 2007,
    "originalDuration": 195,
    "duration": 193,
    "cueIn": 0,
    "cutFromEnd": 0,
    "elapsed": 190
  },
  "nextTrack": {
    "start": 1784838459,
    "id": "uLn3yyvsygu2LctGhPaB3L",
    "ytid": "CFg6amMLd-o",
    "title": "Bugatti",
    "artist": "Tiga",
    "year": 2014,
    "originalDuration": 198,
    "duration": 196,
    "cueIn": 0,
    "cutFromEnd": 0
  }
}
```

## Notes on Sync Behavior

- The app follows a radio-style timeline (no manual seek/skip UI).
- It periodically polls the API and can force-resync with r.
- Playback position tracking prefers mpv position when available and falls back to API elapsed timing.

## Troubleshooting

### mpv or yt-dlp not found

If startup fails with a missing prerequisite error, install both binaries and verify:

```bash
which mpv
which yt-dlp
```

### No audio

- Confirm system audio output is active
- Check local volume and hmtv volume controls
- Try resync with r
- Ensure outbound network access to YouTube

### App exits unexpectedly

Run with backtrace enabled:

```bash
RUST_BACKTRACE=1 cargo run
```

## Project Structure

```text
src/
	main.rs        # app bootstrap, terminal lifecycle, async event loop
	api.rs         # HumanMusic.tv API client and polling strategy
	player/mod.rs  # mpv process + IPC control thread
	app.rs         # central app state and update logic
	action.rs      # action/message enum
	event.rs       # key event mapping
	ui/
		mod.rs       # UI dispatch by screen
		ascii.rs     # ASCII assets + helper formatters
		screen1.rs   # Minimal view
		screen2.rs   # Standard view
		screen3.rs   # Rich view
		screen4.rs   # Full view
```

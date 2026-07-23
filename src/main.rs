mod action;
mod api;
mod app;
mod event;
mod player;
mod ui;

use std::io::Stdout;
use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::EventStream;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::{mpsc, watch};

use action::Action;
use app::App;
use player::{PlayerCommand, PlayerStatus};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    if let Err(msg) = player::check_prerequisites() {
        eprintln!("hmtv: {msg}");
        std::process::exit(1);
    }

    install_panic_hook();

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal).await;
    restore_terminal(&mut terminal)?;

    result
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    // mpv backend: plain std::mpsc command channel in, tokio watch status
    // channel out (the mpv thread is not async).
    let (player_cmd_tx, player_cmd_rx) = std::sync::mpsc::channel::<PlayerCommand>();
    let (status_tx, mut status_rx) = watch::channel(PlayerStatus::default());
    let player_thread = player::spawn_player_thread(player_cmd_rx, status_tx);

    // Single async action channel fed by: the API poller, the player-status
    // bridge task below, and the terminal event loop.
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

    let api_task = tokio::spawn(api::run_api_poller(action_tx.clone()));

    let status_bridge_tx = action_tx.clone();
    let status_task = tokio::spawn(async move {
        // Send the initial status immediately, then forward every change.
        loop {
            let status = status_rx.borrow().clone();
            if status_bridge_tx
                .send(Action::PlayerStatusChanged(status))
                .is_err()
            {
                return;
            }
            if status_rx.changed().await.is_err() {
                return;
            }
        }
    });

    let mut app = App::new(player_cmd_tx.clone());
    let mut events = EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(120));

    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, &app))?;

        tokio::select! {
            _ = tick_interval.tick() => {
                app.update(Action::Tick);
            }
            maybe_event = events.next() => {
                if let Some(Ok(event)) = maybe_event
                    && let Some(action) = event::handle_event(event) {
                    app.update(action);
                }
            }
            Some(action) = action_rx.recv() => {
                app.update(action);
            }
        }
    }

    api_task.abort();
    status_task.abort();
    let _ = player_cmd_tx.send(PlayerCommand::Shutdown);
    let _ = player_thread.join();

    Ok(())
}

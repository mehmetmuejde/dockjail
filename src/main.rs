mod display;
mod dock;
mod ffi;
mod launch_agent;
mod logging;
mod permission;
mod tap;

use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};

/// Keeps the macOS Dock locked to the display it is on.
#[derive(Parser)]
#[command(name = "dockjail", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List connected displays with the index to use for `on`
    Displays,
    /// Check the accessibility permission and ask for it if missing
    Check,
    /// Lock the Dock to a display, immediately and at every login
    On {
        /// Index from `dockjail displays`; default: the display this terminal window is on
        index: Option<usize>,
    },
    /// Unlock the Dock and stop locking at login
    Off,
    /// Print the last lines of the login agent's log
    Log {
        /// Number of lines to show
        #[arg(short = 'n', long, default_value_t = 50)]
        lines: usize,
    },
    /// Lock the Dock to the given display in the foreground (used by the login item)
    #[command(hide = true)]
    Run {
        /// UUID of the display the Dock stays on
        display: String,
    },
}

/// How long `on` waits before it checks that the agent survived its start.
const AGENT_START_WAIT: Duration = Duration::from_secs(3);

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Displays => displays(),
        Command::Check => check(),
        Command::On { index } => on(index),
        Command::Off => off(),
        Command::Log { lines } => log(lines),
        Command::Run { display } => run(display),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

/// For commands that need the permission: never asks, only points to `dockjail check`.
fn require_permission() -> Result<(), String> {
    if permission::is_trusted() {
        return Ok(());
    }
    Err(format!("accessibility permission missing. Run `dockjail check` first.\n{}", permission::terminal_hint()))
}

fn check() -> Result<(), String> {
    if !permission::request() {
        return Err(format!("accessibility permission missing.\n{}", permission::terminal_hint()));
    }
    println!("accessibility permission granted to {}", permission::terminal_name());
    println!("next: `dockjail on` locks the Dock. The login agent it starts is a separate program and asks for");
    println!("its own permission; `dockjail on` tells you if it is missing.");
    Ok(())
}

fn displays() -> Result<(), String> {
    let dock = dock::current_display();
    let default = default_display();

    println!("{:<6} {:<38} {:<22} TYPE", "INDEX", "UUID", "FRAME");
    for (index, d) in display::list().iter().enumerate() {
        let mut kind = vec![if d.is_builtin { "built-in" } else { "external" }];
        if d.is_main {
            kind.push("main");
        }
        if dock == Some(d.id) {
            kind.push("dock");
        }
        if default == Some(d.id) {
            kind.push("default");
        }
        let (origin, size) = (d.frame.origin, d.frame.size);
        let frame = format!("{},{} {}x{}", origin.x, origin.y, size.width, size.height);
        println!("{:<6} {:<38} {:<22} {}", index, d.uuid, frame, kind.join(", "));
    }
    Ok(())
}

fn on(index: Option<usize>) -> Result<(), String> {
    require_permission()?;
    let target = match index {
        Some(index) => display::list()
            .get(index)
            .map(|d| d.id)
            .ok_or_else(|| format!("no display with index {index}, see `dockjail displays`"))?,
        None => default_display().ok_or("cannot determine a display, pick one with `dockjail on INDEX`")?,
    };
    let uuid = display::uuid_of(target);

    let path = launch_agent::install(std::slice::from_ref(&uuid))?;
    println!("installed {}", path.display());

    // The agent is a separate program and needs its own Accessibility entry. Without it, it exits at once.
    std::thread::sleep(AGENT_START_WAIT);
    if !launch_agent::is_running() {
        permission::open_settings();
        return Err(format!(
            "the login agent is not running yet (System Settings just opened).\n{}",
            permission::agent_hint()
        ));
    }
    println!("Dock locked to display {uuid}");
    // dockjail never moves the Dock itself, it only keeps it from leaving.
    if dock::current_display().is_some_and(|dock| dock != target) {
        println!(
            "the Dock is still on another display: push the cursor against the bottom edge of this one to move it here"
        );
    }
    Ok(())
}

/// The display `on` locks to without an index: the one the command was typed on.
fn default_display() -> Option<u32> {
    display::of_front_window().or_else(display::under_cursor).or_else(dock::current_display)
}

fn off() -> Result<(), String> {
    println!("{}", if launch_agent::uninstall()? { "Dock unlocked" } else { "Dock was not locked" });
    Ok(())
}

fn log(lines: usize) -> Result<(), String> {
    let path = launch_agent::log_path()?;
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("no log yet ({})", path.display());
            return Ok(());
        }
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    let all: Vec<&str> = text.lines().collect();
    for line in &all[all.len().saturating_sub(lines)..] {
        println!("{line}");
    }
    Ok(())
}

fn run(display: String) -> Result<(), String> {
    // Started by launchd, dockjail is its own responsible process: asking (not just checking) is what
    // adds it to the Accessibility list, so the user has an entry to enable.
    if !permission::request() {
        return Err(format!("accessibility permission missing.\n{}", permission::agent_hint()));
    }
    logging::log!("locking the Dock to display {display}");
    tap::run(display)
}

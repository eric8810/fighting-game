use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use enigo::{Button, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use xcap::Window;

const DEFAULT_WINDOW: &str = "Tickle Fighting Engine";

#[derive(Parser)]
#[command(name = "game-test")]
#[command(about = "CLI tool for testing Tickle Fighting Engine")]
struct Cli {
    /// Target window title (partial match)
    #[arg(long, default_value = DEFAULT_WINDOW)]
    window: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture a screenshot of the game window
    Screenshot {
        /// Output file path
        #[arg(short, long, default_value = "screenshot.png")]
        output: PathBuf,
    },
    /// Send keyboard input to the game
    Input {
        #[command(subcommand)]
        kind: InputKind,
    },
    /// Focus the game window
    Focus,
    /// List all visible windows
    Windows,
}

#[derive(Subcommand)]
enum InputKind {
    /// Press a single key
    Key {
        /// Key name: left, right, up, down, a-z, enter, escape, space, f1-f12
        key: String,
        /// Hold duration in milliseconds
        #[arg(long, default_value = "50")]
        hold: u64,
    },
    /// Send a comma-separated sequence of keys
    Sequence {
        /// e.g. "left,left,a" or "down,right,a"
        keys: String,
        /// Delay between each key in milliseconds
        #[arg(long, default_value = "100")]
        interval: u64,
    },
}

fn find_window(title: &str) -> Result<Window> {
    let windows = Window::all().context("Failed to enumerate windows")?;
    windows
        .into_iter()
        .find(|w| w.title().contains(title))
        .with_context(|| format!("Window '{}' not found. Is the game running?", title))
}

fn parse_key(s: &str) -> Result<Key> {
    Ok(match s.to_lowercase().as_str() {
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "enter" | "return" => Key::Return,
        "escape" | "esc" => Key::Escape,
        "space" => Key::Space,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        k if k.len() == 1 => Key::Unicode(k.chars().next().unwrap()),
        _ => anyhow::bail!("Unknown key '{}'. Use: left/right/up/down/enter/escape/space/f1-f12/a-z/0-9", s),
    })
}

#[cfg(windows)]
fn focus_window(title: &str) -> Result<()> {
    // Use xcap to find window (supports partial match), then click it to focus
    let window = find_window(title)?;

    let mut enigo = Enigo::new(&Settings::default())
        .context("Failed to initialize input system")?;

    // Click window center to focus it
    let center_x = window.x() + (window.width() as i32 / 2);
    let center_y = window.y() + (window.height() as i32 / 2);

    enigo.move_mouse(center_x, center_y, enigo::Coordinate::Abs)
        .context("Failed to move mouse")?;
    enigo.button(enigo::Button::Left, Direction::Click)
        .context("Failed to click")?;

    thread::sleep(Duration::from_millis(150));
    Ok(())
}

#[cfg(not(windows))]
fn focus_window(_title: &str) -> Result<()> {
    anyhow::bail!("focus_window is only supported on Windows")
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Screenshot { output } => {
            let window = find_window(&cli.window)?;
            let image = window
                .capture_image()
                .context("Failed to capture screenshot")?;
            image
                .save(&output)
                .with_context(|| format!("Failed to save to {:?}", output))?;
            println!("Screenshot saved: {:?} ({}x{})", output, image.width(), image.height());
        }

        Commands::Input { kind } => {
            focus_window(&cli.window)?;

            let mut enigo = Enigo::new(&Settings::default())
                .context("Failed to initialize input system")?;

            match kind {
                InputKind::Key { key, hold } => {
                    let k = parse_key(&key)?;
                    enigo.key(k, Direction::Press).context("key press failed")?;
                    thread::sleep(Duration::from_millis(hold));
                    enigo.key(k, Direction::Release).context("key release failed")?;
                    println!("Key: {} (held {}ms)", key, hold);
                }

                InputKind::Sequence { keys, interval } => {
                    for key_str in keys.split(',') {
                        let key_str = key_str.trim();
                        let k = parse_key(key_str)?;
                        enigo.key(k, Direction::Click).context("key click failed")?;
                        println!("Key: {}", key_str);
                        thread::sleep(Duration::from_millis(interval));
                    }
                }
            }
        }

        Commands::Focus => {
            focus_window(&cli.window)?;
            println!("Focused: {}", cli.window);
        }

        Commands::Windows => {
            let windows = Window::all().context("Failed to enumerate windows")?;
            for w in windows.iter().filter(|w| !w.title().is_empty()) {
                println!("[{}] \"{}\" {}x{}", w.id(), w.title(), w.width(), w.height());
            }
        }
    }

    Ok(())
}

mod app;
mod engine;
mod events;
mod theme;
mod ui;

use crate::app::{App, AppOptions};
use crate::engine::{GameDifficulty, GameOptions};
use crate::events::{AppEvent, Events};
use crate::theme::Theme;

use clap::Parser;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use std::time::Duration;
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser, Debug)]
#[clap(about = "Wordlet is a command line Wordle clone.", version, author)]
struct Args {
    #[clap(
        short,
        long,
        default_value = "easy",
        help = "Change the game's difficulty. Valid values are easy and hard"
    )]
    difficulty: String,

    #[clap(
        short,
        long,
        default_value = "dark",
        help = "Change the display colors. Valid values are light and dark"
    )]
    theme: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let args = Args::parse();
    let difficulty = match args.difficulty.as_ref() {
        "hard" => GameDifficulty::Hard,
        _ => GameDifficulty::Easy,
    };

    let theme = match args.theme.as_ref() {
        "light" => Theme::light_theme(),
        _ => Theme::dark_theme(),
    };

    let mut app = App::new(AppOptions {
        theme: theme,
        game_config: GameOptions {
            answer: None,
            difficulty: difficulty,
        },
    });

    let tick_rate = Duration::from_millis(100);
    let events = Events::new(tick_rate);

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            let _r = ui::draw(frame, &mut app);
        })?;

        match events.next()? {
            AppEvent::Input(event) => app.on_key(event),
            AppEvent::Tick => {}
        }

        if app.should_quit {
            disable_raw_mode()?;
            terminal.clear()?;
            terminal.show_cursor()?;
            break;
        }
    }

    Ok(())
}

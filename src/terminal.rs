use crossterm::{
    event::{self, Event, KeyCode::Char, KeyCode},
    execute,
    style::{Color, SetBackgroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{widgets::List as RList};
use ratatui::{widgets::ListItem as RListItem};
use textwrap;
use pandoculation;
use std::collections::HashMap;

use crate::input::*;
use crate::session::*;
use crate::app::{App};

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

pub fn start_interface(input: &Input) -> Result<Session> {
    log::trace!("In start_interface");

    startup()?;

    let result = run(input);

    shutdown()?;

    result
}

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stdout(), SetBackgroundColor(Color::Reset));
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn run(input: &Input) -> Result<Session> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut app = App::new();
    app.load_input(input);

    let background_hex = app.color_palette.background_hex.clone();
    let color: Color = parse_hex_color(&background_hex).expect("Could not parse hex colour code");

    execute!(
        std::io::stdout(),
        SetBackgroundColor(color),
        Clear(ClearType::All)
    );

    loop {
        t.draw(|f| {
            f.render_widget(&mut app, f.size());
        });

        update(&mut app)?;

        if app.should_quit {
            break;
        }
    }

    Ok(app.get_session())
}

fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    Char('q') => app.quit(),
                    Char('g') => app.display_items.start(),
                    Char('G') => app.display_items.end(),
                    Char('j') => app.display_items.next(),
                    Char('k') => app.display_items.previous(),
                    KeyCode::Enter => {
                        app.deeper();
                    },
                    KeyCode::Backspace => {
                        app.higher();
                    },
                    KeyCode::Tab => {
                        app.farther();
                    },
                    KeyCode::Esc => {
                        app.closer();
                    },
                    _ => {},
                }
            }
        }
    }

    Ok(())
}

fn parse_hex_color(hex_color_str: &str) -> Result<Color> {
    let hex_color_str = if hex_color_str.starts_with('#') {
        &hex_color_str[1..]
    } else {
        hex_color_str
    };

    let hex_color = u32::from_str_radix(hex_color_str, 16)?;

    Ok(Color::Rgb {
        r: ((hex_color >> 16) & 0xFF) as u8,
        g: ((hex_color >> 8) & 0xFF) as u8,
        b: (hex_color & 0xFF) as u8,
    })
}

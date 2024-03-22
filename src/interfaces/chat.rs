use serde_json::Value;
use crossterm::{
    event::{self, Event, KeyCode::Char, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{widgets::List as RList};
use ratatui::{widgets::ListItem as RListItem};
use linked_hash_map::LinkedHashMap;
use webbrowser;
use pandoculation;
use crate::models;

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

struct App {
    should_quit: bool,
    chat: Option<pandoculation::Chat>,
    session_result: Option<models::session::Session>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            chat: None,
            session_result: None,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn load_chat(&mut self, chat: &pandoculation::Chat) {
        self.chat = Some(chat.clone());
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {

    }
}

pub fn start(chat: &pandoculation::Chat) -> Result<Option<models::session::Session>> {
    log::trace!("In start");

    startup()?;
    
    let result = run(chat);

    shutdown()?;

    match result {
        Ok(result) => Ok(result),
        Err(error) => {
            log::error!("Error: {:?}", error);
            Err("An error occurring while running chat interface".into())
        }
    }
}

fn startup() -> Result<()> {
  enable_raw_mode()?;
  execute!(std::io::stderr(), EnterAlternateScreen)?;
  Ok(())
}

fn shutdown() -> Result<()> {
  execute!(std::io::stderr(), LeaveAlternateScreen)?;
  disable_raw_mode()?;
  Ok(())
}

fn run(chat: &pandoculation::Chat) -> Result<Option<models::session::Session>> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut app = App::new();
    app.load_chat(chat);

    loop {
        t.draw(|f| {
            f.render_widget(&mut app, f.size());
        });

        update(&mut app)?;

        if app.should_quit {
            break;
        }
    }

    if let Some(session_result) = app.session_result {
        Ok(Some(session_result))
    } else {
        Ok(None)
    }
}

fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    Char('q') => app.should_quit = true,
                    Char('j') => {
                    }
                    Char('k') => {
                    }
                    KeyCode::Enter => {
                    },
                    KeyCode::Esc => {
                    },
                    _ => {},
                }
            }
        }
    }

    Ok(())
}

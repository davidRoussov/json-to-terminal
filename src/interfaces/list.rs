use std::io::{self, stdout};

use serde_json::Value;
use crossterm::{
    event::{self, Event, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

struct App {
  counter: i64,
  should_quit: bool,
}

pub fn start_list_interface(json: Value) -> Result<()> {
    log::trace!("In start_list_interface");
    startup()?;
    let status = run(json.clone());
    shutdown()?;
    status?;
    Ok(())
}

fn run(json: Value) -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut app = App { counter: 0, should_quit: false };

    loop {
        t.draw(|f| {
            ui(&app, f);
        });

        update(&mut app)?;

        if app.should_quit {
            break;
        }
    }

    Ok(())
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

fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    Char('j') => app.counter += 1,
                    Char('k') => app.counter -= 1,
                    Char('q') => app.should_quit = true,
                    _ => {},
                }
            }

        }
    }
    Ok(())
}

fn ui(app: &App, frame: &mut Frame) {

    let current_tab: usize = 0;

    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(4),
            Constraint::Min(0),
        ],
    )
    .split(frame.size());

    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Document"),
        main_layout[0],
    );

    let tabs = Tabs::new(vec!["Tab1", "Tab2", "Tab3", "Tab4"])
        .block(Block::default().title("Lists").borders(Borders::ALL))
        .style(Style::default().white())
        .highlight_style(Style::default().yellow())
        .select(current_tab)
        .divider(symbols::DOT);

    frame.render_widget(
        tabs,
        main_layout[1],
    );
}

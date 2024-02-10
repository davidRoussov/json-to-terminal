use std::io::{self, stdout};

use serde_json::Value;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

pub fn start_list_interface(json: Value) -> io::Result<()> {
    log::trace!("In start_list_interface");

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame) {
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

    frame.render_widget(
        Tabs::new(vec!["Tab1", "Tab2", "Tab3", "Tab4"])
            .block(Block::default().title("Lists").borders(Borders::ALL))
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(0)
            .divider(symbols::DOT)
            .padding("->", "<-"),
        main_layout[1],
    );
}

use crossterm::{
    event::{self, Event, KeyCode::Char, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{widgets::List as RList};
use ratatui::{widgets::ListItem as RListItem};
use textwrap;

use crate::input::{Input};
use crate::session::{Session};

pub struct App {
    pub should_quit: bool,
    pub session: Session,
    input: Option<Input>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            input: None,
            session: Session {
                result: "init".to_string()
            }
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn load_input(&mut self, input: &Input) {

    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {

    }
}

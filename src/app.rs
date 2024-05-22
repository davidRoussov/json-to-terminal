use crossterm::{
    event::{self, Event, KeyCode::Char, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{widgets::List as RList};
use ratatui::{widgets::ListItem as RListItem};
use textwrap;

use crate::input::{Input, ComplexObject};
use crate::session::{Session};

pub struct App {
    pub should_quit: bool,
    pub session: Session,
    input: Option<Input>,
    current_depth: u16,
    display_items: StatefulList<ComplexObject>,
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    last_selected: Option<usize>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            input: None,
            display_items: StatefulList::<ComplexObject>::with_items(Vec::new()),
            session: Session {
                result: "init".to_string()
            },
            current_depth: 0,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn load_input(&mut self, input: &Input) {
        self.input = Some(input.clone());

        let complex_objects = self.input
            .clone()
            .unwrap()
            .complex_objects
            .iter()
            .filter(|item| {
                item.depth == self.current_depth
            })
            .cloned()
            .collect();

        self.display_items = StatefulList::<ComplexObject>::with_items(complex_objects);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(0),
        ]);

        let [header_area, body_area] = vertical.areas(area);

        self.render_document_header(header_area, buf);
        self.render_body(body_area, buf);
    }
}

impl App {
    fn render_document_header(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Placeholder document title")
            .block(Block::default().borders(Borders::ALL).title("Document"))
            .render(area, buf);
    }

    fn render_body(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<RListItem> = self
            .display_items
            .items
            .iter()
            .map(|item| {

                let mut lines: Vec<Line> = Vec::new();

                let span = Span::styled(
                    "test".to_string(),
                    Style::new()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green)
                );

                lines.push(span.into());

                RListItem::new(lines)
            })
            .collect();

        let list = RList::new(items.clone())
            .block(Block::default().title("List").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        StatefulWidget::render(list, area, buf, &mut self.display_items.state);
    }

}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: items,
            last_selected: None,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }
}

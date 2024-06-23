use crossterm::{
    event::{self, Event, KeyCode::Char, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{widgets::List as RList};
use ratatui::{widgets::ListItem as RListItem};
use textwrap;
use std::collections::HashMap;
use std::str::FromStr;

use crate::input::{Input};
use crate::session::{Session};

const DEFAULT_DEPTH: usize = 1;
const DEFAULT_RANGE: usize = 3;

const DEFAULT_PRIMARY_COLOR_HEX: &str = "#00FF00"; // green
const DEFAULT_SECONDARY_COLOR_HEX: &str = "#FFFFFF"; // white
const DEFAULT_BACKGROUND_COLOR_HEX: &str = "#000011"; // black

pub struct ColorPalette {
    pub primary_hex: String,
    pub secondary_hex: String,
    pub background_hex: String,
}

pub struct App {
    pub should_quit: bool,
    pub session: Session,
    pub display_items: StatefulList<ComplexObject>,
    pub color_palette: ColorPalette,
    current_depth: usize,
    current_range: usize,
    input: Option<Input>,
}

type ComplexObject = Input;

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    last_selected: Option<usize>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            display_items: StatefulList::<ComplexObject>::with_items(Vec::new()),
            session: Session {
                depth: DEFAULT_DEPTH,
            },
            current_depth: DEFAULT_DEPTH,
            current_range: DEFAULT_RANGE,
            color_palette: ColorPalette {
                primary_hex: DEFAULT_PRIMARY_COLOR_HEX.to_string(),
                secondary_hex: DEFAULT_SECONDARY_COLOR_HEX.to_string(),
                background_hex: DEFAULT_BACKGROUND_COLOR_HEX.to_string(),
            },
            input: None,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn deeper(&mut self) {
        self.current_depth = self.current_depth + 1;
        self.init_display_items();
    }

    pub fn higher(&mut self) {
        if self.current_depth > 0 {
            self.current_depth = self.current_depth - 1;
            self.init_display_items();
        }
    }
    
    pub fn closer(&mut self) {
        if self.current_range > 0 {
            self.current_range = self.current_range - 1;
            self.init_display_items();
        }
    }

    pub fn farther(&mut self) {
        self.current_range = self.current_range + 1;
        self.init_display_items();
    }

    pub fn get_session(&self) -> Session {
        Session {
            depth: self.current_depth,
        }
    }

    pub fn load_input(&mut self, input: &Input) {
        self.input = Some(input.clone());
        self.init_display_items();
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

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn start(&mut self) {
        self.state.select(Some(0));
    }

    pub fn end(&mut self) {
        self.state.select(Some(self.items.len() - 1));
    }
}

impl App {
    fn init_display_items(&mut self) {
        let mut results = Vec::new();
        self.input
            .clone()
            .unwrap()
            .go_down_depth(
                self.current_depth,
                &mut results
            );

        self.display_items = StatefulList::<ComplexObject>::with_items(results);
    }

    fn get_current_object(&mut self) -> Option<ComplexObject> {
        if let Some(i) = self.display_items.state.selected() {
            Some(self.display_items.items[i].clone())
        } else {
            None
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
        ]);

        let [header_area, body_area] = vertical.areas(area);

        self.render_header(header_area, buf);
        self.render_body(body_area, buf);
    }
}

impl App {
    fn render_header(&mut self, area: Rect, buf: &mut Buffer) {
        let text_color: Color = Color::from_str("#111111").unwrap();

        let span: Span = Span::styled(
            "Document title".to_string(),
            Style::new()
                .fg(text_color)

        ).into();

        let color: Color = Color::from_str(&self.color_palette.primary_hex).unwrap();

        Paragraph::new(span)
            .block(
                Block::default()
                    .style(
                        Style::default()
                            .bg(color)
                    )
            )
            .render(area, buf);
    }

    fn render_body(&mut self, area: Rect, buf: &mut Buffer) {
        let text_color: Color = Color::from_str(&self.color_palette.secondary_hex).unwrap();
        let background_color: Color = Color::from_str(&self.color_palette.background_hex).unwrap();

        let items: Vec<RListItem> = self.display_items.items
            .clone()
            .iter()
            .map(|item| {
                let mut text = String::new();
                item.to_string(0, &mut text);
                text
            })
            .filter(|item| {
                !item.is_empty()
            })
            .map(|item| {
                let mut lines: Vec<Line> = Vec::new();

                let mut wrapped_string = textwrap::wrap(&item, &textwrap::Options::new(160));

                if wrapped_string.len() > 20 {
                    wrapped_string.truncate(20);
                }
                for segment in wrapped_string.iter() {
                    let span: Span = Span::styled(
                        segment.to_string(),
                        Style::new()
                            .fg(text_color)
                            .bg(background_color)
                    ).into();
                    lines.push(Line::from(span));
                }

                lines.push(
                    Line::from("".to_string())
                );

                RListItem::new(lines)

            })
            .collect();

        let list = RList::new(items)
            .block(
                Block::new()
                    .borders(Borders::NONE)
                    .padding(Padding::vertical(1))
                    .border_style(
                         Style::new()
                             .bg(background_color)
                    )
            )
            .highlight_symbol(">")
            .repeat_highlight_symbol(false)
            .direction(ListDirection::TopToBottom);

        StatefulWidget::render(list, area, buf, &mut self.display_items.state);
    }
}

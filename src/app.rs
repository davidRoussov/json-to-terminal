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

use crate::input::{Input, Content};
use crate::session::{Session};

const DEFAULT_DEPTH: usize = 1;

//const DEFAULT_PRIMARY_COLOR_HEX: &str = "#00FF00"; // green
//const DEFAULT_SECONDARY_COLOR_HEX: &str = "#FFFFFF"; // white
//const DEFAULT_BACKGROUND_COLOR_HEX: &str = "#000011"; // black

const DEFAULT_PRIMARY_COLOR_HEX: &str = "#FF6600";
const DEFAULT_SECONDARY_COLOR_HEX: &str = "#828282";
const DEFAULT_BACKGROUND_COLOR_HEX: &str = "#F6F6EF";

pub struct ColorPalette {
    pub primary_hex: String,
    pub secondary_hex: String,
    pub background_hex: String,
}

pub struct App {
    pub should_quit: bool,
    pub should_display_primary_content: bool,
    pub session: Session,
    pub display_items: StatefulList<ComplexObject>,
    pub color_palette: ColorPalette,
    current_depth: usize,
    input: Option<Input>,
    current_value_index: usize,
    current_value: Option<String>,
}

type ComplexObject = Content;

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    last_selected: Option<usize>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            should_display_primary_content: true,
            display_items: StatefulList::<ComplexObject>::with_items(Vec::new()),
            session: Session {
                depth: DEFAULT_DEPTH,
                value: None,
            },
            current_depth: DEFAULT_DEPTH,
            color_palette: ColorPalette {
                primary_hex: DEFAULT_PRIMARY_COLOR_HEX.to_string(),
                secondary_hex: DEFAULT_SECONDARY_COLOR_HEX.to_string(),
                background_hex: DEFAULT_BACKGROUND_COLOR_HEX.to_string(),
            },
            input: None,
            current_value_index: 0,
            current_value: None,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    
    pub fn toggle_primary_content(&mut self) {
        self.should_display_primary_content = !self.should_display_primary_content;
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
    
    pub fn get_session(&self) -> Session {
        Session {
            depth: self.current_depth,
            value: self.current_value.clone(),
        }
    }

    pub fn load_input(&mut self, input: &Input) {
        self.input = Some(input.clone());
        self.init_display_items();
    }

    pub fn first_value(&mut self) {
        self.current_value_index = 0;
    }

    pub fn next_value(&mut self) {
        self.current_value_index += 1;
    }

    pub fn previous_value(&mut self) {
        if self.current_value_index > 0 {
            self.current_value_index -= 1;
        }
    }

    pub fn exit_without_value(&mut self) {
        self.session.value = None;
        self.quit();
    }

    pub fn exit_with_value(&mut self) {
        self.session.value = self.current_value.clone();
        self.quit();
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
            .data
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

        let text = "Placeholder title";

        let span: Span = Span::styled(
            text,
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
        let main_content_color: Color = Color::from_str("#111111").unwrap();
        let text_color: Color = Color::from_str(&self.color_palette.secondary_hex).unwrap();
        let background_color: Color = Color::from_str(&self.color_palette.background_hex).unwrap();

        let items: Vec<RListItem> = self.display_items.items
            .clone()
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let mut lines: Vec<Line> = Vec::new();

                item.to_lines(
                    &self.should_display_primary_content,
                    &main_content_color,
                    &text_color,
                    &background_color,
                    &mut lines,
                    0,
                );

                if let Some(selected_item_index) = self.display_items.state.selected() {
                    if selected_item_index == index {
                        let mut current_index = 0;
                        for line in lines.iter_mut() {
                            for span in line.spans.iter_mut() {
                                if !span.content.trim().is_empty() {
                                    if self.current_value_index == current_index {
                                        self.current_value = Some(span.content.clone().to_string());
                                        let color = Color::from_str("#00FF00").unwrap();
                                        span.style = span.style.bg(color);
                                    }
                                    current_index += 1;
                                }
                            }
                        }
                    }
                }

                if lines.len() > 0 {
                    lines.push(
                        Line::from("".to_string())
                    );
                }

                lines.truncate(30);

                lines
            })
            .filter(|item| {
                item.len() > 0
            })
            .map(|item| {
                RListItem::new(item)
            })
            .collect();

        let list = RList::new(items)
            .block(
                Block::new()
                    .borders(Borders::NONE)
                    .padding(Padding::vertical(1))
                    .style(
                        Style::new()
                            .fg(text_color)
                            .bg(background_color)
                    )
            )
            .highlight_symbol(">")
            .repeat_highlight_symbol(false)
            .direction(ListDirection::TopToBottom);

        StatefulWidget::render(list, area, buf, &mut self.display_items.state);
    }
}

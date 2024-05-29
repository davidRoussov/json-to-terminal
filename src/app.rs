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

use crate::input::{Input, ComplexType, ComplexObject};
use crate::session::{Session};

pub struct App {
    pub should_quit: bool,
    pub session: Session,
    pub display_items: StatefulList<ComplexObject>,
    complex_types: Vec<ComplexType>,
    complex_objects: Vec<ComplexObject>,
    current_depth: u16,
    selected_parents: Vec<String>,
    max_depth: u16,
}

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    last_selected: Option<usize>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            complex_types: Vec::new(),
            complex_objects: Vec::new(),
            display_items: StatefulList::<ComplexObject>::with_items(Vec::new()),
            session: Session {
                result: "init".to_string()
            },
            current_depth: 1,
            selected_parents: Vec::new(),
            max_depth: 0,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn deeper(&mut self) {
        if let Some(current_object) = self.get_current_object() {
            if self.current_depth < self.max_depth {
                self.current_depth = self.current_depth + 1;
                self.selected_parents.push(current_object.id.clone());
                self.init_display_items();

                if self.display_items.items.len() == 1 {
                    self.display_items.state.select(Some(0));
                    self.deeper();
                }
            }
        }
    }

    pub fn higher(&mut self) {
        if self.current_depth > 0 {
            self.selected_parents.pop();
            self.current_depth = self.current_depth - 1;
            self.init_display_items();
        }
    }

    pub fn load_input(&mut self, input: &Input) {

        // TODO For some reason parversion produces empty objects 
        // We filter them out here, but should investigate root cause
        self.complex_objects = input.complex_objects
            .clone()
            .iter()
            .filter(|item| {
                !(item.values.is_empty() && item.complex_objects.is_empty())
            })
            .cloned()
            .collect();
        self.complex_types = input.complex_types
            .clone();

        self.init_display_items();
        self.update_max_depth();
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
}

impl App {
    fn update_max_depth(&mut self) {
        self.max_depth = self.complex_objects
            .iter()
            .fold(0, |acc, item| if item.depth > acc { item.depth } else { acc });
    }

    fn init_display_items(&mut self) {
        let complex_objects: Vec<ComplexObject> = self.complex_objects
            .iter()
            .filter(|item| {
                if item.depth != self.current_depth {
                    return false;
                }

                if !self.selected_parents.is_empty() {
                    if let Some(parent_id) = &item.parent_id {
                        if parent_id != self.selected_parents.last().unwrap() {
                            return false;
                        }
                    }
                }

                true
            })
            .cloned()
            .collect();

        self.display_items = StatefulList::<ComplexObject>::with_items(complex_objects);
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
        let items: Vec<RListItem> = self.display_items.items
            .clone()
            .iter()
            .map(|item| {
                let mut lines: Vec<Line> = Vec::new();
                let complex_string = complex_object_to_string(item.clone(), &self.complex_objects);
                let complex_type = self.complex_types.iter().find(|t| t.id == item.type_id).unwrap();
                let mut wrapped_string = textwrap::wrap(&complex_string, &textwrap::Options::new(160));
                let mut truncated = false;

                let title_span: Span = Span::styled(
                    complex_type.name.to_string(),
                    Style::new()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Blue)
                ).into();
                lines.push(Line::from(title_span));

                if wrapped_string.len() > 20 {
                    wrapped_string.truncate(20);
                    truncated = true;
                }
                
                for segment in wrapped_string.iter() {
                    let span: Span = Span::styled(
                        segment.to_string(),
                        Style::new()
                            .fg(Color::Green)
                    ).into();
                    lines.push(Line::from(span));
                }

                if truncated {
                    let span: Span = Span::styled(
                        " (truncated)",
                        Style::new()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Red)
                    ).into();
                    lines.push(Line::from(span));
                }

                RListItem::new(lines)
            })
            .collect();

        let list = RList::new(items)
            .block(Block::default().title("List").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        StatefulWidget::render(list, area, buf, &mut self.display_items.state);
    }
}

fn complex_object_to_string(complex_object: ComplexObject, complex_objects: &Vec<ComplexObject>) -> String {
    let mut result: String = complex_object.values
        .values()
        .enumerate()
        .fold(String::new(), |mut acc, (index, item)| {
            if item.get("is_url").unwrap() == "true" || item.get("is_decorative").unwrap() == "true" || item.get("is_id").unwrap() == "true" {
                acc
            } else {
                acc + " " + item.get("value").unwrap().trim()
            }
        });

    for id in complex_object.complex_objects.iter() {
        let child_object = complex_objects
            .iter()
            .find(|item| item.id == *id);

        if let Some(child_object) = child_object {
            result.push_str(
                &complex_object_to_string(child_object.clone(), complex_objects)
            );
        }
    }

    result
}

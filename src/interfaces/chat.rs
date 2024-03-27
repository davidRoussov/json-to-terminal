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
use textwrap;
use pandoculation;
use std::collections::HashMap;
use crate::models;

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

#[derive(Debug, Default)]
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    last_selected: Option<usize>,
}

struct App {
    should_quit: bool,
    chat: Option<pandoculation::Chat>,
    display_items: StatefulList<pandoculation::ChatItem>,
    session_result: Option<models::session::Session>,
    item_height_map: HashMap<String, u16>,
    item_collapse_map: HashMap<String, bool>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            chat: None,
            session_result: None,
            display_items: StatefulList::<pandoculation::ChatItem>::with_items(Vec::new()),
            item_height_map: HashMap::new(),
            item_collapse_map: HashMap::new(),
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn load_chat(&mut self, chat: &pandoculation::Chat) {

        let mut items = chat.clone().items;

        let mut parent_map: HashMap<Option<String>, Vec<pandoculation::ChatItem>> = HashMap::new();
        for item in items.iter().cloned() {
            parent_map.entry(item.data.parent_id.clone()).or_insert_with(Vec::new).push(item);
        }

        fn build_sorted_list(id: Option<String>, parent_map: &HashMap<Option<String>, Vec<pandoculation::ChatItem>>, sorted_list: &mut Vec<pandoculation::ChatItem>, item_height_map: &mut HashMap<String, u16>) {
            if let Some(children) = parent_map.get(&id) {
                for child in children {

                    if let Some(ref some_id) = id {
                        if let Some(h) = item_height_map.get(some_id) {
                            item_height_map.insert(child.data.id.clone(), h + 1);
                        }
                    } else {
                        item_height_map.insert(child.data.id.clone(), 0);
                    }


                    sorted_list.push(child.clone());
                    build_sorted_list(Some(child.data.id.clone()), parent_map, sorted_list, item_height_map);
                }
            }
        }

        let mut item_height_map: HashMap<String, u16> = HashMap::new();
        let mut sorted_items = Vec::new();
        build_sorted_list(None, &parent_map, &mut sorted_items, &mut item_height_map);

        log::debug!("item_height_map: {:?}", item_height_map);

        self.chat = Some(chat.clone());
        self.display_items = StatefulList::<pandoculation::ChatItem>::with_items(sorted_items);
        self.item_height_map = item_height_map;
    }

    pub fn toggleCollapse(&mut self) {
        let selected_item: Option<pandoculation::ChatItem> = if let Some(i) = self.display_items.state.selected() {
            Some(self.display_items.items[i].clone())
        } else {
            None
        };

        if let Some(selected_item) = selected_item {
            let collapsed: Option<bool> = self.item_collapse_map.get(&selected_item.data.id).copied();

            if let Some(false) | None = collapsed {
                self.item_collapse_map.insert(selected_item.data.id, true);
            } else {
                self.item_collapse_map.insert(selected_item.data.id, false);
            }
        }
    }
}

impl App {
    fn render_document_header(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Placeholder document title")
            .block(Block::default().borders(Borders::ALL).title("Document"))
            .render(area, buf);
    }

    fn render_body(&mut self, area: Rect, buf: &mut Buffer) {
        let chat = if let Some(chat) = &self.chat {
            chat
        } else {
            return;
        };

        let vertical_scroll = 0;

        let items: Vec<RListItem> = chat
            .items
            .iter()
            .map(|item| {

                let item_height = self.item_height_map.get(&item.data.id).unwrap_or(&0);
                let whitespace: String = std::iter::repeat(' ').take((*item_height as usize) * 4).collect();

                let mut lines: Vec<Line> = Vec::new();

                let mut line1_spans: Vec<Span> = Vec::new();

                let span_one = Span::styled(format!("{}{}", whitespace, item.data.author), Style::default().fg(Color::Blue));
                line1_spans.push(span_one);

                if let Some(timestamp) = &item.data.timestamp {
                    let span = Span::styled(format!(" {}", timestamp), Style::default().fg(Color::Green));
                    line1_spans.push(span);
                }

                lines.push(
                    Line::from(line1_spans)
                );


                let is_collapsed = self.item_collapse_map.get(&item.data.id);

                if let Some(false) | None = is_collapsed {
                    let wrapped_lines = textwrap::wrap(&item.data.text, &textwrap::Options::new(120));

                    for wrapped_line in wrapped_lines.iter() {
                        lines.push(
                            Line::from(format!("{}{}", whitespace, wrapped_line))
                        );
                    }
                }

                return RListItem::new(lines);
            })
            .collect();


        let list = RList::new(items.clone())
            .block(Block::default().title("Chat").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        StatefulWidget::render(list, area, buf, &mut self.display_items.state);
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
                        app.display_items.next();
                    }
                    Char('k') => {
                        app.display_items.previous();
                    }
                    KeyCode::Enter => {
                    },
                    KeyCode::Esc => {
                    },
                    Char(' ') => {
                        app.toggleCollapse();
                    },
                    _ => {},
                }
            }
        }
    }

    Ok(())
}

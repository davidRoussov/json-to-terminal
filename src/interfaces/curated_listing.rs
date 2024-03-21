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

#[derive(Debug, Default)]
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    last_selected: Option<usize>,
}

struct App {
    should_quit: bool,
    focus_item: bool,
    curated_listing: Option<pandoculation::CuratedListing>,
    display_items: StatefulList<pandoculation::CuratedListingItem>,
    active_item_urls: StatefulList<String>,
    session_result: Option<models::session::Session>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            focus_item: false,
            curated_listing: None,
            display_items: StatefulList::<pandoculation::CuratedListingItem>::with_items(Vec::new()),
            active_item_urls: StatefulList::<String>::with_items(Vec::new()),
            session_result: None,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn load_curated_listing(&mut self, curated_listing: pandoculation::CuratedListing) {
        self.curated_listing = Some(curated_listing.clone());
        self.display_items = StatefulList::<pandoculation::CuratedListingItem>::with_items(curated_listing.items.clone());
    }

    pub fn load_active_item_urls(&mut self, curated_listing_item: pandoculation::CuratedListingItem) {

        let mut urls: Vec<String> = Vec::new();

        urls.push(curated_listing_item.data.url);

        if let Some(chat_url) = &curated_listing_item.data.chat_url {
            urls.push(chat_url.to_string());
        }
       
        self.active_item_urls = StatefulList::<String>::with_items(urls.clone());
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

                let line_one = Line::from(format!("{}", item.data.title))
                    .style(Style::new().add_modifier(Modifier::BOLD));
                let line_two = Line::from(format!("{}", item.data.url))
                    .style(Style::new().yellow());

                lines.push(line_one);
                lines.push(line_two);
               
                if let Some(chat_url) = &item.data.chat_url {
                    let span_one = Span::styled(
                        "chat: ",
                        Style::new()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    );
                    let span_two = Span::styled(
                        format!("{}", chat_url),
                        Style::new()
                            .fg(Color::Yellow)
                    );
                    let line = Line::from(vec![span_one, span_two]);
                    lines.push(line);
                }

                let mut additional_info: Vec<Span> = Vec::new();

                if let Some(points) = &item.data.points {
                    let span_one = Span::styled(
                        "points: ",
                        Style::new()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    );
                    additional_info.push(span_one);

                    let span_two = Span::styled(
                        format!("{}", points),
                        Style::new()
                            .fg(Color::Green)
                    );
                    additional_info.push(span_two);
                }

                if let Some(author) = &item.data.author {
                    let span_one = Span::styled(
                        " author: ",
                        Style::new()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    );
                    additional_info.push(span_one);

                    let span_two = Span::styled(
                        format!("{}", author),
                        Style::new()
                            .fg(Color::Green)
                    );
                    additional_info.push(span_two);
                }

                if let Some(timestamp) = &item.data.timestamp {
                    let span_one = Span::styled(
                        " timestamp: ",
                        Style::new()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    );
                    additional_info.push(span_one);

                    let span_two = Span::styled(
                        format!("{}", timestamp),
                        Style::new()
                            .fg(Color::Green)
                    );
                    additional_info.push(span_two);
                }

                lines.push(Line::from(additional_info));
                
                return RListItem::new(lines);
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

    fn render_footer(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<RListItem> = self
            .active_item_urls
            .items
            .iter()
            .map(|item| {
                let line = Line::from(format!("{}", item))
                    .style(Style::new().yellow());

                return RListItem::new(line);
            })
            .collect();

        let list = RList::new(items.clone())
            .block(Block::default().title("URLs").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);

        StatefulWidget::render(list, area, buf, &mut self.active_item_urls.state);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let length_footer_area = if self.focus_item {
            (self.active_item_urls.items.len() as u16) + 2
        } else {
            0
        };

        let vertical = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(length_footer_area),
        ]);

        let [header_area, body_area, footer_area] = vertical.areas(area);

        self.render_document_header(header_area, buf);
        self.render_body(body_area, buf);
        self.render_footer(footer_area, buf);
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

pub fn start(curated_listing: &pandoculation::CuratedListing) -> Result<Option<models::session::Session>> {
    log::trace!("In start");

    startup()?;
    
    let result = run(curated_listing.clone());

    shutdown()?;

    match result {
        Ok(result) => Ok(result),
        Err(error) => {
            log::error!("Error: {:?}", error);
            Err("An error occurring while running curated listing interface".into())
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

fn run(curated_listing: pandoculation::CuratedListing) -> Result<Option<models::session::Session>> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut app = App::new();
    app.load_curated_listing(curated_listing);

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
                        if app.focus_item {
                            app.active_item_urls.next();
                        } else {
                            app.display_items.next();
                        }
                    }
                    Char('k') => {
                        if app.focus_item {
                            app.active_item_urls.previous();
                        } else {
                            app.display_items.previous();
                        }
                    }
                    KeyCode::Enter => {
                        if app.focus_item {

                            let selected_url: Option<String> = if let Some(i) = app.active_item_urls.state.selected() {
                                Some(app.active_item_urls.items[i].clone())
                            } else {
                                None
                            };

                            if let Some(selected_url) = selected_url {
                                log::debug!("selected_url: {}", selected_url);

                                let session_result = models::session::Session {
                                    url: selected_url,
                                };

                                app.session_result = Some(session_result);
                                app.should_quit = true;
                            }

                        } else {
                            app.focus_item = true;

                            let selected_item: Option<pandoculation::CuratedListingItem> = if let Some(i) = app.display_items.state.selected() {
                                Some(app.display_items.items[i].clone())
                            } else {
                                None
                            };

                            if let Some(selected_item) = selected_item {
                                app.load_active_item_urls(selected_item);
                            }
                        }
                    },
                    KeyCode::Esc => {
                        app.focus_item = false;
                    },
                    _ => {},
                }
            }
        }
    }

    Ok(())
}

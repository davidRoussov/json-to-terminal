use std::io::{self, stdout};

use serde_json::Value;
use crossterm::{
    event::{self, Event, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{widgets::List as RList};
use ratatui::{widgets::ListItem as RListItem};
use linked_hash_map::LinkedHashMap;

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

#[derive(Debug, Default, Clone)]
struct ListItem {
    data: LinkedHashMap<String, String>
}

#[derive(Debug, Default, Clone)]
struct List {
    items: Vec<ListItem>,
}

#[derive(Debug, Default, Clone)]
struct DisplayItem {
    title: String,
    description: String,
}

#[derive(Debug, Default)]
struct StatefulList {
    state: ListState,
    items: Vec<DisplayItem>,
    last_selected: Option<usize>,
}

impl StatefulList {
    fn with_items(items: Vec<DisplayItem>) -> StatefulList {
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

#[derive(Debug, Default)]
struct App {
  should_quit: bool,
  lists: Vec<List>,
  display_items: StatefulList,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            lists: Vec::new(),
            display_items: StatefulList::with_items(Vec::new()),
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn load_json(&mut self, json: Value) {
        self.lists = Vec::new();

        let Some(all_lists) = json.as_array() else {
            panic!("JSON is not an array");
        };

        for json_list in all_lists.iter() {
            let mut list = List {
                items: Vec::new(),
            };

            let Some(json_list_items) = json_list["items"].as_array() else {
                panic!("JSON list items is not an array");
            };

            for json_item in json_list_items {
                let mut list_item = ListItem {
                    data: LinkedHashMap::new(),
                };

                let Some(data_object) = json_item["data"].as_object() else {
                    panic!("JSON item data is not an object");
                };

                for (serde_key, serde_value) in data_object.iter() {
                    let serde_value = serde_value.as_str().expect("Failed to convert to str");
                    let serde_value = serde_value.trim_matches('"');
                    let serde_value = serde_value.to_string();

                    list_item.data.insert(serde_key.to_string(), serde_value);
                }

                list.items.push(list_item);
            }

            self.lists.push(list);
        }

        self.generate_display_items();
    }

    fn generate_display_items(&mut self) {
        if self.lists.len() < 1 {
            panic!("We do not have any lists");
        }

        let first_list = &self.lists[0];

        let display_items: Vec<DisplayItem> = first_list.items.iter().map(|item| {
            DisplayItem {
                title: item.data.get("title").expect("Failed to get title").to_string(),
                description: "test description".to_string(),
            }
        }).collect();

        self.display_items = StatefulList::with_items(display_items);
    }
}

impl Widget for &mut App {

    fn render(self, area: Rect, buf: &mut Buffer) {


        self.render_list(area, buf);
        
    }
}

impl App {

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {

        let items: Vec<RListItem> = self
            .display_items
            .items
            .iter()
            .map(|item| {
                RListItem::new(Line::from(format!("{}", item.title)))
            })
            .collect();

        let mut state = ListState::default();
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

    let mut app = App::default();
    app.load_json(json);

    loop {
        t.draw(|f| {
            f.render_widget(&mut app, f.size());
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
                    Char('q') => app.should_quit = true,
                    Char('j') => app.display_items.next(),
                    Char('k') => app.display_items.previous(),
                    _ => {},
                }
            }

        }
    }
    Ok(())
}

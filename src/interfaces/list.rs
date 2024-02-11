use std::io::{self, stdout};

use serde_json::Value;
use crossterm::{
    event::{self, Event, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use linked_hash_map::LinkedHashMap;

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

#[derive(Debug, Default)]
struct ListItem {
    data: LinkedHashMap<String, String>
}

#[derive(Debug, Default)]
struct List {
    items: Vec<ListItem>,
}

#[derive(Debug, Default)]
struct App {
  should_quit: bool,
  lists: Vec<List>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
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
                    list_item.data.insert(serde_key.to_string(), serde_value.to_string());
                }

                list.items.push(list_item);
            }

            self.lists.push(list);
        }

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
                    Char('q') => app.should_quit = true,
                    _ => {},
                }
            }

        }
    }
    Ok(())
}

fn ui(app: &App, frame: &mut Frame) {
    //let main_layout = Layout::new(
    //    Direction::Vertical,
    //    [
    //        Constraint::Length(4),
    //        Constraint::Min(0),
    //    ],
    //)
    //.split(frame.size());

    //frame.render_widget(
    //    Block::new().borders(Borders::TOP).title("Document"),
    //    main_layout[0],
    //);

    frame.render_widget(
        Paragraph::new(format!(
            "
            {:?}
            ",
            app.lists
          ))
          .block(
            Block::default()
              .title("Lists")
              .title_alignment(Alignment::Center)
              .borders(Borders::ALL)
              .border_type(BorderType::Rounded),
          )
          .style(Style::default().fg(Color::Yellow))
          .alignment(Alignment::Center),
        frame.size()
  );
}

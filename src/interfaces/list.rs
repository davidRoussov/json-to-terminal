use crossterm::{
    ExecutableCommand,
    QueueableCommand,
    terminal,
    cursor,
    style::{
        self,
        Stylize,
        Color,
        Attribute,
        SetBackgroundColor,
        SetForegroundColor
    },
    cursor::position,
    event::{
        poll,
        read,
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode
    },
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode
    },
    Result,
};
use std::io::{self, Write};
use serde_json::Value;
use std::{io::stdout, time::Duration, time::Instant};
use uuid::Uuid;
use linked_hash_map::LinkedHashMap;

#[derive(Clone)]
struct Line {
    text: String,
    id: String,
    data: LinkedHashMap<String, String>,
}

pub fn start_list_interface(stdout: &mut io::Stdout, json: Value) -> Result<()> {
    log::trace!("In start_list_interface");

    let size = terminal::size()?;
    let terminal_y = &size.1;
    let terminal_x = &size.0;
    log::debug!("Terminal dimensions: {} x {}", terminal_x, terminal_y);

    let padding_x: u16 = 1;
    let padding_y: u16 = 1;
    log::debug!("padding_x: {}, padding_y: {}", padding_x, padding_y);

    let lines = get_lines(&json, *terminal_y, *terminal_x, padding_y, padding_x);

    let mut offset: usize = 0;
    let mut page: Vec<Line> = get_page(&lines, *terminal_y, padding_y, offset);
    let mut current_item_id = page[0].id.clone();

    print_page_to_screen(stdout, padding_x, padding_y, page, current_item_id.clone());

    let mut last_char = None;
    let mut last_time = Instant::now();

    loop {
        if poll(Duration::from_millis(1_000))? {
            let event = read()?;

            if event == Event::Key(KeyCode::Char('q').into()) {
                break;
            }

            if event == Event::Key(KeyCode::Char('j').into()) {
                clear_screen(stdout);

                current_item_id = get_next_item_id(&lines, &current_item_id.clone());
                page = get_page(&lines, *terminal_y, padding_y, offset);
                print_page_to_screen(stdout, padding_x, padding_y, page, current_item_id.clone());

                last_char = Some('j');
                last_time = Instant::now();
            }

            if event == Event::Key(KeyCode::Char('k').into()) {
                clear_screen(stdout);

                current_item_id = get_previous_item_id(&lines, &current_item_id.clone());
                page = get_page(&lines, *terminal_y, padding_y, offset);
                print_page_to_screen(stdout, padding_x, padding_y, page, current_item_id.clone());

                last_char = Some('k');
                last_time = Instant::now();
            }

            if event == Event::Key(KeyCode::Char('g').into()) {

                if last_char == Some('g') && last_time.elapsed() < Duration::from_millis(500) {
                    clear_screen(stdout);
                }

                last_char = Some('g');
                last_time = Instant::now();
            }

            if event == Event::Key(KeyCode::Char('G').into()) {
                clear_screen(stdout);

                last_char = Some('G');
                last_time = Instant::now();
            }
        }
    }

    Ok(())
}


fn clear_screen(stdout: &mut io::Stdout) -> Result<()> {
    let size = terminal::size()?;

    for y in 0..size.1 {
        for x in 0..size.0 {
            stdout
                .queue(cursor::MoveTo(x,y))?
                .queue(style::PrintStyledContent( "█".white()))?;
        }
    }
    stdout.flush()?;

    Ok(())
}

fn print_page_to_screen(
    stdout: &mut io::Stdout,
    padding_x: u16,
    padding_y: u16,
    page: Vec<Line>,
    current_item_id: String,
) -> Result<()> {
    log::trace!("In print_page_to_screen");

    let mut x = padding_x;
    let mut y = padding_y;

    for line in page.iter() {
        let mut z = x;

        for (key, value) in line.data.iter() {
            let label = format!("{}: ", key);
            let content = format!("{} ", value);

            let background = if &current_item_id == &line.id { Color::Yellow } else { Color::White };

            stdout
                .queue(cursor::MoveTo(z, y))?
                .queue(style::PrintStyledContent(
                    label.clone()
                    .with(Color::Black)
                    .on(background)
                    .attribute(Attribute::Bold)
                ))?;

            z += label.len() as u16;

            stdout
                .queue(cursor::MoveTo(z, y))?
                .queue(style::PrintStyledContent(
                    content.clone()
                    .with(Color::Black)
                    .on(background)
                ))?;

            z += content.len() as u16;

            stdout.flush()?;
        }

        y += 1;
    }

    Ok(())
}

fn get_page(lines: &Vec<Line>, terminal_y: u16, padding_y: u16, offset: usize) -> Vec<Line> {
    log::trace!("In get_page");

    let max_lines = (terminal_y - 2 * padding_y) as usize;
    log::debug!("max_lines: {}", max_lines);

    return lines[offset..max_lines + offset].to_vec();
}

fn get_lines(
    json: &Value,
    terminal_y: u16, 
    terminal_x: u16,
    padding_y: u16,
    padding_x: u16,
) -> Vec<Line> {
    log::trace!("In get_lines");

    let mut lines: Vec<Line> = Vec::new();

    let Some(items) = json["items"].as_array() else {
        log::debug!("json items is not an array");
        return Vec::new();
    };

    let chunk_size = (terminal_x - 2 * padding_x) as usize;
    log::debug!("chunk_size: {}", chunk_size);

    for item in items.iter() {
        if let Some(obj_map) = item["data"].as_object() {

            let mut sorted_data: Vec<(String, Value)> = obj_map.iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();

            sorted_data.sort_by(|a, b| a.0.cmp(&b.0));


            let id = Uuid::new_v4().to_string();
            log::debug!("id: {}", id);



            let mut current_line = Line {
                id: id.clone(),
                text: "".to_string(),
                data: LinkedHashMap::new(),
            };

            for (serde_key, serde_value) in sorted_data.iter() {
                log::debug!("serde_key: {}", serde_key);
                log::debug!("serde_value: {}", serde_value);

                let serde_key_str = serde_key;
                let serde_value_str = serde_value.as_str().expect("value is not a string");
                let key = serde_key_str.to_string();
                let value = serde_value_str.to_string();
                let segment = format!("{}: {} ", key, value);

                if segment.len() > chunk_size {
                    log::info!("Segment length is greater than screen width");

                } else if current_line.text.len() + segment.len() < chunk_size {
                    current_line.text += &segment;
                    current_line.data.insert(key, value);

                } else if current_line.text.len() + segment.len() > chunk_size {
                    lines.push(current_line);
                    current_line = Line {
                        id: id.clone(),
                        text: segment,
                        data: LinkedHashMap::new(),
                    };
                    current_line.data.insert(key, value);
                }
                        
            }

            if current_line.text.len() > 0 {
                lines.push(current_line);
                current_line = Line {
                    id: id.clone(),
                    text: "".to_string(),
                    data: LinkedHashMap::new(),
                };
            }
        }

        let blank_line = Line {
            id: "".to_string(),
            text: "".to_string(),
            data: LinkedHashMap::new(),
        };
        lines.push(blank_line);
    }

    return lines;
}

fn get_next_item_id(lines: &Vec<Line>, current_item_id: &String) -> String {
    log::trace!("In get_next_item_id");
    log::debug!("current_item_id: {}", current_item_id);

    let mut found_current_lines = false;
    let mut next_item_id = current_item_id.clone();

    for line in lines.iter() {
        log::debug!("id: {}", &line.id);

        let id = &line.id;

        if id == "" {
            continue;
        }

        if id == current_item_id {
            found_current_lines = true;
        } else if id != current_item_id && found_current_lines {
            next_item_id = id.clone();
            break;
        }
    }

    return next_item_id;
}

fn get_previous_item_id(lines: &Vec<Line>, current_item_id: &String) -> String {
    log::trace!("In get_previous_item_id");
    log::debug!("current_item_id: {}", current_item_id);

    let mut found_current_lines = false;
    let mut next_item_id = current_item_id.clone();

    for line in lines.iter().rev() {
        log::debug!("id: {}", &line.id);

        let id = &line.id;

        if id == "" {
            continue;
        }

        if id == current_item_id {
            found_current_lines = true;
        } else if id != current_item_id && found_current_lines {
            next_item_id = id.clone();
            break;
        }
    }

    return next_item_id;
}

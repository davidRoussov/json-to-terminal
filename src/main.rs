extern crate serde;
extern crate serde_json;
extern crate simple_logging;
extern crate log;

use std::io::{self, Write};
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
use std::{io::stdout, time::Duration, time::Instant};
use std::process;
use std::env;
use std::io::{Read};
use std::fs::File;
use serde_json::Value;
use log::{LevelFilter};
use std::fs::OpenOptions;
use clap::{Arg, App};
use atty::Stream;

mod interfaces {
    pub mod list;
}

#[derive(Clone)]
struct Line {
    post_id: String,
    text: String,
}

fn get_json_from_file(file_name: &str) -> String {
    let mut file = File::open(file_name).unwrap_or_else(|err| {
        eprintln!("Failed to open file: {}", err);
        process::exit(1);
    });
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|err| {
        eprintln!("Failed to read file: {}", err);
        process::exit(1);
    });

    return contents;
}

fn setup() -> Result<io::Stdout> {
    let mut stdout = io::stdout();

    simple_logging::log_to_file("debug.log", LevelFilter::Trace);

    enable_raw_mode()?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(terminal::EnterAlternateScreen);
    stdout.execute(SetBackgroundColor(Color::White));
    stdout.execute(SetForegroundColor(Color::Black));

    let size = terminal::size()?;
    log::debug!("Terminal dimensions: {} x {}", &size.0, &size.1);

    for y in 0..size.1 {
        for x in 0..size.0 {
            stdout
                .queue(cursor::MoveTo(x,y))?
                .queue(style::PrintStyledContent("█".white()))?;
        }
    }
    stdout.flush()?;

    return Ok(stdout);
}

fn cleanup(mut stdout: io::Stdout) {
    disable_raw_mode();

    stdout.execute(terminal::LeaveAlternateScreen);
}

fn simple_chunk(s: &str, chunk_size: usize) -> Vec<String> {
    s.chars()
        .collect::<Vec<char>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

fn chunk_string(s: &str, chunk_size: usize) -> Vec<String> {
     assert!(chunk_size > 0, "chunk_size must be greater than zero");

     let mut chunks = Vec::new();
     let mut char_indices = s.char_indices().peekable();

     while let Some((start_index, _)) = char_indices.peek().cloned() {
         let mut end_index = start_index;
         let mut chunk = String::new();
         let mut last_is_whitespace = false;

         while chunk.chars().count() < chunk_size {
             match char_indices.next() {
                 Some((idx, ch)) => {
                     if !ch.is_whitespace() {
                         last_is_whitespace = false;
                     }
                     end_index = idx;
                     chunk.push(ch);
                     if ch.is_whitespace() {
                         last_is_whitespace = true;
                     }
                 },
                 None => break,
             }
         }

         // If the last character is not whitespace and the next character exists
         // and it is not whitespace, we end this chunk early.
         if !last_is_whitespace && char_indices.peek().map_or(false, |&(_, ch_next)|
 !ch_next.is_whitespace()) {
             // We need to find the start of the next word and trim the current chunk up to that point.
             while let Some((idx, ch)) = char_indices.next() {
                 // Once we find a whitespace or end of string, we stop and set the end_index before the start of next word
                 if ch.is_whitespace() {
                     end_index = idx;
                     break;
                 }
             }
         }

         // Taking a slice to avoid breaking a character between chunks
         chunks.push(s[start_index..end_index].to_string());

         // continue processing from the next character, keeping last_is_whitespace accurate
         if let Some((_, ch)) = char_indices.peek() {
             last_is_whitespace = ch.is_whitespace();
         }
     }

     chunks
 }

fn get_line_from_string(text: String, line_length: usize) -> String {
    let diff = line_length - (text.chars().count());
    let line = text + &" ".repeat(diff);

    return line;
}

fn flatten_json(json: Value, line_length: u16) -> Vec<String> {
    let mut vec: Vec<String> = Vec::new();

    let Some(chapters) = json["chapters"].as_array() else {
        log::debug!("chapter not array");
        return Vec::new();
    };

    for chapter in chapters.iter() {
        vec.push(get_line_from_string(serde_json::to_string(&chapter["title"]).unwrap(), line_length as usize));

        let Some(sections) = chapter["sections"].as_array() else {
            log::debug!("sections not array");
            return Vec::new();
        };

        for section in sections.iter() {
            vec.push(get_line_from_string(serde_json::to_string(&section["title"]).unwrap(), line_length as usize));

            let Some(content) = section["content"].as_array() else {
                log::debug!("content not array");
                return Vec::new();
            };

            for paragraph in content.iter() {
                let paragraphString = serde_json::to_string(paragraph).unwrap();
                let paragraph_without_quotes = paragraphString.trim_matches('\"');
                let indentedParagraph = " ".repeat(2) + &paragraph_without_quotes;
                let chunks = chunk_string(&indentedParagraph, line_length.into());

                for (index, chunk) in chunks.iter().enumerate() {
                    if index == chunks.len() - 1 {
                        vec.push(get_line_from_string(chunk.to_string(), line_length as usize));
                    } else {
                        vec.push(chunk.to_string());
                    }
                }
            }
        }
    }

    return vec;
}

fn print_to_screen(stdout: &mut io::Stdout, vec: &mut Vec<String>, line_length: u16, start_page: usize, paddingX: u16, paddingY: u16, page_right_margin: u16, page_height: u16) -> Result<()> {
    let size = terminal::size()?;

    let pagesPerScreen: u16 = (size.0 - (2 * paddingX)) / (line_length + page_right_margin);
    log::debug!("Pages per screen: {}", pagesPerScreen);

    let mut x = paddingX;

    for pagePerScreen in 0..=(pagesPerScreen - 1) {

        let page: usize = start_page + (pagePerScreen as usize);
        log::debug!("page: {}", page);

        let offset = page * (page_height as usize);
        let mut end = offset + (page_height as usize);


        if end >= vec.len() {
            end = vec.len();
        }

        if offset >= end {
            break;
        }

        let currentLines = &vec[offset..end];

        let mut y = paddingY;

        for line in currentLines.iter() {
            stdout
                .queue(cursor::MoveTo(x, y))?
                .queue(style::PrintStyledContent(
                    line.clone()
                    .with(Color::Black)
                    .on(Color::White)
                ))?;

            y += 1;
        }

        x += line_length + page_right_margin;
        log::debug!("x: {}", x);
    }
    stdout.flush()?;

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

fn load_stdin() -> io::Result<String> {
    log::trace!("In load_stdin");

    if atty::is(Stream::Stdin) {
        return Err(io::Error::new(io::ErrorKind::Other, "stdin not redirected"));
    }
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    return Ok(buffer);
}

fn escape_for_terminal(s: &str) -> String {
     s.replace('\x1b', "\\x1b")  // Escape ANSI escape code start
      .replace('\n', "\\n")      // Escape newline
      .replace('\r', "\\r")      // Escape carriage return
      .replace('\t', "\\t")      // Escape tab
      // Add more replacements as needed
 }

fn get_lines(
    json: &Value,
    terminal_y: u16, 
    terminal_x: u16,
    padding_y: u16,
    padding_x: u16,
) -> Vec<Line> {
    log::trace!("In get_lines");

    let Some(items) = json.as_array() else {
        log::debug!("json is not an array");
        return Vec::new();
    };
    let chunk_size = (terminal_x - 2 * padding_x) as usize;

    let lines: Vec<Line> = items.iter().flat_map(|item| {
        let id = serde_json::to_string(&item["id"]).unwrap();
        let content = serde_json::to_string(&item["content"]).unwrap();
        let strings: Vec<String> = chunk_string(&content, chunk_size);

        let mut current_lines: Vec<Line> = strings
            .iter()
            .map(|s| Line {
                post_id: id.clone(),
                text: s.clone(),
            })
            .collect();

        current_lines.push(Line {
            post_id: "".to_string(),
            text: "".to_string(),
        });

        return current_lines;
    }).collect();

    return lines;
}

fn get_page(lines: &Vec<Line>, terminal_y: u16, padding_y: u16, offset: usize) -> Vec<Line> {
    log::trace!("In get_page");

    let max_lines = (terminal_y - 2 * padding_y) as usize;
    log::debug!("max_lines: {}", max_lines);

    return lines[offset..max_lines + offset].to_vec();
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

        let background = if &current_item_id == &line.post_id { Color::Yellow } else { Color::White };

        stdout
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent(
                line.text.clone()
                .with(Color::Black)
                .on(background)
            ))?;

        y += 1;
    }

    Ok(())
}

fn get_previous_item_id(lines: &Vec<Line>, current_item_id: &String) -> String {
    log::trace!("In get_previous_item_id");
    log::debug!("current_item_id: {}", current_item_id);

    let mut found_current_lines = false;
    let mut next_item_id = current_item_id.clone();

    for line in lines.iter().rev() {
        log::debug!("post_id: {}", &line.post_id);

        let post_id = &line.post_id;

        if post_id == "" {
            continue;
        }

        if post_id == current_item_id {
            found_current_lines = true;
        } else if post_id != current_item_id && found_current_lines {
            next_item_id = post_id.clone();
            break;
        }
    }

    return next_item_id;
}

fn get_next_item_id(lines: &Vec<Line>, current_item_id: &String) -> String {
    log::trace!("In get_next_item_id");
    log::debug!("current_item_id: {}", current_item_id);

    let mut found_current_lines = false;
    let mut next_item_id = current_item_id.clone();

    for line in lines.iter() {
        log::debug!("post_id: {}", &line.post_id);

        let post_id = &line.post_id;

        if post_id == "" {
            continue;
        }

        if post_id == current_item_id {
            found_current_lines = true;
        } else if post_id != current_item_id && found_current_lines {
            next_item_id = post_id.clone();
            break;
        }
    }

    return next_item_id;
}

fn chat_to_terminal(stdout: &mut io::Stdout, json: Value) -> Result<()> {
    log::trace!("In chat_to_terminal");

    let size = terminal::size()?;
    let terminal_y = &size.1;
    let terminal_x = &size.0;
    log::debug!("Terminal dimensions: {} x {}", terminal_x, terminal_y);

    let padding_x: u16 = 1;
    let padding_y: u16 = 1; // does tmux status bar take up one row
    log::debug!("padding_x: {}, padding_y: {}", padding_x, padding_y);

    let lines = get_lines(&json, *terminal_y, *terminal_x, padding_y, padding_x);

    let mut offset: usize = 0;
    let mut page: Vec<Line> = get_page(&lines, *terminal_y, padding_y, offset);
    let mut current_item_id = page[0].post_id.clone();

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

                //offset += 1;
                current_item_id = get_next_item_id(&lines, &current_item_id.clone());
                log::debug!("current_item_id: {}", current_item_id);

                page = get_page(&lines, *terminal_y, padding_y, offset);
                print_page_to_screen(stdout, padding_x, padding_y, page, current_item_id.clone());

                last_char = Some('j');
                last_time = Instant::now();
            }

            if event == Event::Key(KeyCode::Char('k').into()) {
                clear_screen(stdout);

                //offset -= 1;
                current_item_id = get_previous_item_id(&lines, &current_item_id.clone());
                log::debug!("current_item_id: {}", current_item_id);

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

fn main() -> io::Result<()> {
    let mut stdout = setup()?;

    let mut json_string = String::new();

    match load_stdin() {
        Ok(stdin) => {
            json_string = stdin;
        }
        Err(e) => {
            log::debug!("Did not receive input from stdin");
        }
    }

    let matches = App::new("json-to-terminal")
        .arg(Arg::with_name("type")
             .short('t')
             .long("type")
             .value_name("TYPE")
             .required(true))
        .arg(Arg::with_name("file")
             .short('f')
             .long("file")
             .value_name("FILE")
             .help("Provide file as document for processing"))
        .get_matches();

    if let Some(file_name) = matches.value_of("file") {
        log::debug!("file_name: {}", file_name);

        json_string = get_json_from_file(file_name);
    } else {
        log::debug!("File not provided");
    }


    if json_string.trim().is_empty() {
        log::debug!("JSON not provided, aborting...");
        return Ok(());
    }


    log::debug!("{}", json_string);


    let json: Value = serde_json::from_str(&json_string).expect("Failed to parse JSON");


    if let Some(data_type) = matches.value_of("type") {
        log::debug!("data_type: {}", data_type);

        match data_type {
            "chat" => chat_to_terminal(&mut stdout, json)?,
            "list" => interfaces::list::start_list_interface(&mut stdout, json)?,
            _ => log::error!("Unexpected data type: {}", data_type),
        }
    } else {
        log::info!("Data type not provided, aborting...");
    }


    cleanup(stdout);

    Ok(())
}

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
                .queue(style::PrintStyledContent( "█".white()))?;
        }
    }
    stdout.flush()?;

    return Ok(stdout);
}

fn cleanup(mut stdout: io::Stdout) {
    disable_raw_mode();

    stdout.execute(terminal::LeaveAlternateScreen);
}

fn chunk_string(s: &str, chunk_size: usize) -> Vec<String> {
    s.chars()
        .collect::<Vec<char>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect())
        .collect()
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

fn conversation_to_terminal(json: Value) {
    log::trace!("In conversation_to_terminal");
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
            "conversation" => conversation_to_terminal(json),
            _ => log::error!("Unexpected data type: {}", data_type),
        }
    } else {
        log::info!("Data type not provided, aborting...");
    }
















    //let line_length: u16;
    //let mut page_right_margin: u16 = 2;
    //let mut paddingX: u16 = 0;
    //let mut paddingY: u16 = 0;
    //let size = terminal::size()?;

    //if size.0 > 0 && size.0 < 100 {
    //    paddingX = 1;
    //    paddingY = 0;
    //    line_length = size.0 - 2 * paddingX - page_right_margin;
    //} else if size.0 >= 100 && size.0 < 150 {
    //    paddingX = 3;
    //    paddingY = 1;
    //    line_length = (size.0 - 2 * paddingX) / 2 - page_right_margin;
    //} else {
    //    paddingX = 5;
    //    paddingY = 2;
    //    line_length = 60 - page_right_margin;
    //}
    //log::debug!("Padding dimensions: {} x {}", paddingX, paddingY);
    //log::debug!("Line length: {}", line_length);

    //let page_height: u16 = size.1 - 2 * paddingY;
    //log::debug!("Page height: {}", page_height);
    //




    //let args: Vec<String> = env::args().collect();

    //let file_name = &args[1];

    //let json: Value = get_json_from_file(file_name);
    //log::debug!("Title: {}", json["title"]);




    //let mut vec = flatten_json(json, line_length);



    //let total_pages: u16 = (vec.len() as u16) / page_height;
    //log::debug!("Number of pages: {}", total_pages);




    //let mut start_page: usize = 0;




    //print_to_screen(&mut stdout, &mut vec, line_length, start_page, paddingX, paddingY, page_right_margin, page_height);







    //let mut last_char = None;
    //let mut last_time = Instant::now();

    //loop {
    //    if poll(Duration::from_millis(1_000))? {
    //        let event = read()?;

    //        if event == Event::Key(KeyCode::Char('q').into()) {
    //            break;
    //        }

    //        if event == Event::Key(KeyCode::Char('j').into()) {
    //            if start_page < (total_pages as usize) {
    //                clear_screen(&mut stdout);
    //                start_page += 1;
    //                print_to_screen(&mut stdout, &mut vec, line_length, start_page, paddingX, paddingY, page_right_margin, page_height);
    //            }

    //            last_char = Some('j');
    //            last_time = Instant::now();
    //        }

    //        if event == Event::Key(KeyCode::Char('k').into()) {
    //            if start_page > 0 {
    //                clear_screen(&mut stdout);
    //                start_page -= 1;
    //                print_to_screen(&mut stdout, &mut vec, line_length, start_page, paddingX, paddingY, page_right_margin, page_height);
    //            }

    //            last_char = Some('k');
    //            last_time = Instant::now();
    //        }

    //        if event == Event::Key(KeyCode::Char('g').into()) {

    //            if last_char == Some('g') && last_time.elapsed() < Duration::from_millis(500) {
    //                clear_screen(&mut stdout);
    //                start_page = 0;
    //                print_to_screen(&mut stdout, &mut vec, line_length, start_page, paddingX, paddingY, page_right_margin, page_height);
    //            }

    //            last_char = Some('g');
    //            last_time = Instant::now();
    //        }

    //        if event == Event::Key(KeyCode::Char('G').into()) {
    //            clear_screen(&mut stdout);
    //            start_page = total_pages as usize;
    //            print_to_screen(&mut stdout, &mut vec, line_length, start_page, paddingX, paddingY, page_right_margin, page_height);

    //            last_char = Some('G');
    //            last_time = Instant::now();
    //        }
    //    }
    //}

    //cleanup(stdout);

    Ok(())
}

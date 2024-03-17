extern crate simple_logging;
extern crate log;

use async_recursion::async_recursion;
use tokio::runtime::Runtime;
use std::io::{self, Write};
use std::process;
use std::env;
use std::io::{Read};
use std::fs::File;
use log::{LevelFilter};
use clap::{Arg, App};
use atty::Stream;

pub mod interfaces;
pub mod models;

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

fn load_stdin() -> io::Result<String> {
    log::trace!("In load_stdin");

    if atty::is(Stream::Stdin) {
        return Err(io::Error::new(io::ErrorKind::Other, "stdin not redirected"));
    }
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    return Ok(buffer);
}

fn main() -> io::Result<()> {
    let _ = simple_logging::log_to_file("debug.log", LevelFilter::Trace);

    let mut json_string = String::new();

    match load_stdin() {
        Ok(stdin) => {
            json_string = stdin;
        }
        Err(e) => {
            log::debug!("Did not receive input from stdin");
        }
    }

    let matches = App::new("tooey")
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

    let result = tooey::json_to_terminal(json_string);

    match result {
        Ok(session_result) => {
            if let Some(session_result) = session_result {
                println!("{:?}", session_result);
            } 
        }
        Err(err) => {
            log::error!("session ended in error: {:?}", err);
        }
    }

    Ok(())
}

use std::io::{self};
use std::process;
use std::io::{Read};
use std::fs::File;
use log::{LevelFilter};
use clap::{Arg, App};
use atty::Stream;
use env_logger::Builder;

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

fn init_logging() -> Builder {
    let mut builder = Builder::from_default_env();

    builder.filter(None, LevelFilter::Off); // disables all logging
    builder.filter_module("parversion", LevelFilter::Trace);

    let log_file = std::fs::File::create("./debug/debug.log").unwrap();
    builder.target(env_logger::Target::Pipe(Box::new(log_file)));

    builder.init();

    builder
}

fn main() -> io::Result<()> {
    let _ = init_logging();

    let mut json_string = String::new();

    match load_stdin() {
        Ok(stdin) => {
            json_string = stdin;
        }
        Err(_e) => {
            log::debug!("Did not receive input from stdin");
        }
    }

    let matches = App::new("tooey")
        .arg(Arg::with_name("file")
             .short('f')
             .long("file")
             .value_name("FILE")
             .help("Provide processed document as file"))
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

    let result = tooey::render(json_string);

    match result {
        Ok(session_result) => {
            println!("{:?}", session_result);
        }
        Err(err) => {
            log::error!("session ended in error: {:?}", err);
        }
    }

    Ok(())
}

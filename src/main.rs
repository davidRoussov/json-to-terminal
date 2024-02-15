extern crate serde;
extern crate serde_json;
extern crate simple_logging;
extern crate log;

use async_recursion::async_recursion;
use tokio::runtime::Runtime;
use std::io::{self, Write};
use std::process;
use std::env;
use std::io::{Read};
use std::fs::File;
use serde_json::Value;
use log::{LevelFilter};
use clap::{Arg, App};
use atty::Stream;

mod interfaces {
    pub mod list;
}
mod models {
    pub mod session;
}
mod utilities;

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

async fn get_json(session: &models::session::Session, json: Option<Value>) -> Value {
    log::trace!("In get_json");

    if let Some(json) = json {
        return json;
    } else {
        let document = utilities::get_document(session.url.clone()).await.unwrap();
        println!("document: {}", document);

        panic!("Get json unimplemented");
    }
}

#[async_recursion::async_recursion(?Send)]
async fn start_session(session: &models::session::Session, json: Option<Value>) {
    log::trace!("In start_session");

    let json = get_json(session, json).await;

    match session.content_type.as_str() {
        "list" => {
            match interfaces::list::start_list_interface(json) {
                Ok(session_result) => {
                    if let Some(session_result) = session_result {
                        start_session(&session_result, None).await;
                    }
                }
                Err(_) => {
                    log::error!("List session ended with error");
                }
            }
        }
        _ => {}
    }
}

fn main() -> io::Result<()> {
    log::trace!("In main");

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

    let mut session = models::session::Session {
        url: "".to_string(),
        content_type: "".to_string(),
    };

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        if let Some(data_type) = matches.value_of("type") {
            log::debug!("data_type: {}", data_type);

            session.content_type = data_type.to_string();
            start_session(&session, Some(json)).await;
        } else {
            log::info!("Data type not provided, aborting...");
        }
    });

    Ok(())
}

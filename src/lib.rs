use serde_json::Value;
use serde::{Serialize};

pub mod interfaces;
pub mod models;

#[derive(Debug)]
#[derive(Clone)]
#[derive(Serialize)]
pub enum Errors {
    JsonNotProvided,
    UnexpectedDocumentType,
    UnexpectedError,
    IncorrectParser,
}

pub fn json_to_terminal(json_string: String, document_type: &str) -> Result<Option<models::session::Session>, Errors> {
    log::trace!("In json_to_terminal");
    log::debug!("json_string: {:?}", json_string);

    let json: Value = serde_json::from_str(&json_string).expect("Failed to parse JSON");

    match document_type {
        "list" => {
            if let Ok(session_result) = interfaces::list::start_list_interface(json) {
                Ok(session_result)
            } else {
                Err(Errors::UnexpectedError)
            }
        }
        _ => {
            Err(Errors::UnexpectedDocumentType)
        }
    }
}

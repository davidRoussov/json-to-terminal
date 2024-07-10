mod error;
mod terminal;
mod input;
mod session;
mod app;

use error::{Errors};
use terminal::{start_interface};
use input::{Input};
use session::{Session};

pub fn render(json: String) -> Result<Session, Errors> {
    log::trace!("In render");
    log::trace!("json: {}", json);

    let input: Input = serde_json::from_str(&json).map_err(|e| {
        log::error!("deserialization error: {}", e);
        Errors::DeserializationError
    })?;

    log::info!("Successfully deserialized JSON");

    start_interface(&input).map_err(|e| {
        log::error!("{}", e);
        Errors::UnexpectedError
    })
}

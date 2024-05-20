mod error;
mod interface;
mod input;
mod session;

use error::{Errors};
use interface::{start_interface};
use input::{Input};
use session::{Session};

pub fn render(json: String) -> Result<Session, Errors> {
    log::trace!("In render");

    let input: Input = serde_json::from_str(&json).map_err(|e| {
        log::error!("{}", e);
        Errors::DeserializationError
    })?;

    log::info!("Successfully deserialized JSON");

    start_interface(&input)
}

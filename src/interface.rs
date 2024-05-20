use crate::input::*;
use crate::session::*;
use crate::error::{Errors};

pub fn start_interface(input: &Input) -> Result<Session, Errors> {
    log::trace!("In start_interface");

    Ok(Session {
        result: "all g".to_string()
    })
}

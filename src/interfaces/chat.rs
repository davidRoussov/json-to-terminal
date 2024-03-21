
use pandoculation;
use crate::models;

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

pub fn start(chat: &pandoculation::Chat) -> Result<Option<models::session::Session>> {
    log::trace!("In start");

    panic!("unimplemented");
}

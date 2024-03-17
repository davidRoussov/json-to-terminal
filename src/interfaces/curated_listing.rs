use pandoculation;

use crate::models;

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

pub fn start(curated_listing: &pandoculation::CuratedListing) -> Result<Option<models::session::Session>> {
    log::trace!("In start");

    println!("{:?}", curated_listing);


    let session = models::session::Session {
        url: String::new(),
        content_type: String::new(),
    };

    Ok(Some(session))
}

use serde_json::Value;
use serde::{Serialize};
use std::collections::HashMap;
use pandoculation;

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

enum DeserializationResult {
    CuratedListing(pandoculation::CuratedListing),
}

pub fn json_to_terminal(json_string: String) -> Result<Option<models::session::Session>, Errors> {
    log::trace!("In json_to_terminal");
    log::debug!("json_string: {:?}", json_string);

    match try_deserialize(&json_string) {
         Some(DeserializationResult::CuratedListing(ref listing)) => {
             interfaces::curated_listing::start(listing).map_err(|e| {
                 log::error!("Error: {:?}", e);
                 Errors::UnexpectedError
             })
         },
         None => Err(Errors::UnexpectedDocumentType)
    }
}

fn try_deserialize(data: &str) -> Option<DeserializationResult> {
     if let Ok(listing) = serde_json::from_str::<HashMap<String, pandoculation::CuratedListing>>(data) {

         if let Some(curated_listing) = listing.get("CuratedListing") {
             return Some(DeserializationResult::CuratedListing(curated_listing.clone()));
         }
     }
     // Add more deserialization attempts here for other structs...

     None
 }

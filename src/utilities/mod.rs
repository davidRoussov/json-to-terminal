use reqwest;
use std::io::{Error, ErrorKind};
use std::io::{self};

pub async fn get_document(url: String) -> Result<String, io::Error> {
    log::trace!("In get_document");
    log::debug!("url: {}", url);

    println!("one");

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await;

    match response {
        Ok(success_response) => {
            let text = success_response.text().await.unwrap();
            return Ok(text);
        }
        Err(_err) => {
            return Err(Error::new(ErrorKind::InvalidData, "error"));
        }
    }
}

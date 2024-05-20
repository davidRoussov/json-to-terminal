use serde::{Serialize, Deserialize};
use serde_json::{Value};
use std::collections::{HashMap};

mod error;

use error::{Errors};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComplexType {
    pub id: String,
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComplexObject {
    pub id: String,
    pub type_id: String,
    pub values: HashMap<String, String>,
    pub depth: u16,
    pub complex_objects: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputMeta {
    object_count: u64,
    type_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub complex_types: Vec<ComplexType>,
    pub complex_objects: Vec<ComplexObject>,
    pub meta: InputMeta,
}

pub fn render(json: String) -> Result<String, Errors> {
    log::trace!("In render");

    let input: Input = serde_json::from_str(&json).map_err(|e| {
        log::error!("{}", e);
        Errors::DeserializationError
    })?;

    log::debug!("complex_types: {:?}", input.complex_types);
    log::debug!("complex_objects: {:?}", input.complex_objects);

    unimplemented!()
}

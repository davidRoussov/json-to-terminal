use serde::{Serialize, Deserialize};
use std::collections::{HashMap};

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
    pub object_count: u64,
    pub type_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub complex_types: Vec<ComplexType>,
    pub complex_objects: Vec<ComplexObject>,
    pub meta: InputMeta,
}

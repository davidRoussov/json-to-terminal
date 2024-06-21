use serde::{Serialize, Deserialize};
use std::collections::{HashMap};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub values: HashMap<String, String>,
    pub children: Vec<Input>,
}

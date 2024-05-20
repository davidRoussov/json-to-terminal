use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Errors {
    UnexpectedError,
    DeserializationError,
}

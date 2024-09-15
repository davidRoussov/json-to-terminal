use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
}

pub type History = Vec<HistoryEntry>;

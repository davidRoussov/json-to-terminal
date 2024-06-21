use serde::{Serialize, Deserialize};
use std::collections::{HashMap};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub values: HashMap<String, String>,
    pub children: Vec<Input>,
}

impl Input {
    pub fn go_down_depth(&self, depth: usize, range: usize, results: &mut Vec<HashMap<String, String>>) {
        if depth == 0 {
            self.go_down_range(range, results);
        } else {
            for child in &self.children {
                child.go_down_depth(depth - 1, range, results);
            }
        }
    }

    pub fn go_down_range(&self, range: usize, results: &mut Vec<HashMap<String, String>>) {
        if range == 0 {
            return;
        }

        if !self.values.is_empty() {
            results.push(self.values.clone());
        }

        for child in &self.children {
            child.go_down_range(range - 1, results);
        }
    }
}

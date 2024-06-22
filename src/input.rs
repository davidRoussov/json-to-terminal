use serde::{Serialize, Deserialize};
use std::collections::{HashMap};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub values: HashMap<String, String>,
    pub children: Vec<Input>,
}

impl Input {
    pub fn go_down_depth(&self, depth: usize, results: &mut Vec<Input>) {
        if depth == 0 {
            results.push(self.clone());
        } else {
            for child in &self.children {
                child.go_down_depth(depth - 1, results);
            }
        }
    }

    pub fn to_string(&self, relative_depth: usize, result: &mut String) {
        let indentation = format!("\n{}", " ".repeat(relative_depth * 2));
        let values = self.values.iter()
            .fold(String::new(), |mut acc, (_key, value)| {
                acc.push_str(&format!(" {}", value));
                acc
            });
        let text = format!("{}{}", indentation, values);

        result.push_str(&text);

        for child in &self.children {
            child.to_string(relative_depth + 1, result);
        }
    }
}

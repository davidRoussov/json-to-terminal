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

    pub fn to_string(&self, indentation_factor: usize, result: &mut String) {
        let indentation = format!("\n{}", " ".repeat(indentation_factor));

        let mut keys: Vec<_> = self.values.keys().collect();
        keys.sort();

        let values = keys.iter()
            .fold(String::new(), |mut acc, key| {
                let value = self.values.get(*key).unwrap();
                acc.push_str(&format!(" {}", value));
                acc
            });
        let text = format!("{}{}", indentation, values);

        result.push_str(&text);

        for child in &self.children {
            let new_depth = if values.is_empty() { indentation_factor } else { indentation_factor + 1 };
            child.to_string(new_depth, result);
        }
    }
}

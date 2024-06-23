use serde::{Serialize, Deserialize};
use std::collections::{HashMap};
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputMetadata {
    pub is_id: bool,
    pub is_url: bool,
    pub is_page_link: bool,
    pub is_action_link: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputValue {
    pub meta: InputMetadata,
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub values: Vec<InputValue>,
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
        if !self.values.is_empty() {
            let indentation = format!("{}{}", if !result.is_empty() { "\n" } else { "" }, " ".repeat(indentation_factor));
            let values = self.values.iter()
                .filter(|item| {
                    !item.meta.is_id && !item.meta.is_action_link
                })
                .collect::<Vec<_>>()
                .into_iter()
                .sorted_by(|a, b| {
                    a.name.cmp(&b.name)
                })
                .fold(String::new(), |mut acc, item| {
                    let trimmed = item.value.trim();
                    acc.push_str(&format!(" {}", trimmed));
                    acc
                });
            let text = format!("{}{}", indentation, values);

            result.push_str(&text);
        }

        for child in &self.children {
            let indentation_factor = if self.values.is_empty() { indentation_factor } else { indentation_factor + 1 };
            child.to_string(indentation_factor, result);
        }
    }
}

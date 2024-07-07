use serde::{Serialize, Deserialize};
use std::collections::{HashMap};
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataMetadata {
    pub is_id: bool,
    pub is_url: bool,
    pub is_page_link: bool,
    pub is_action_link: bool,
    pub is_primary_content: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataValue {
    pub meta: DataMetadata,
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    pub values: Vec<DataValue>,
    pub children: Vec<Data>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    pub title: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub data: Data,
    pub metadata: Metadata,
}

impl Data {
    pub fn go_down_depth(&self, depth: usize, results: &mut Vec<Data>) {
        if depth == 0 {
            results.push(self.clone());
        } else {
            for child in &self.children {
                child.go_down_depth(depth - 1, results);
            }
        }
    }

    pub fn to_string(&self, filter_secondary_content: &bool, indentation_factor: usize, result: &mut String) {
        let indentation = format!("{}{}", if !result.is_empty() { "\n" } else { "" }, " ".repeat(indentation_factor * 2));
        let values = self.values.iter()
            .filter(|item| {
                if *filter_secondary_content {
                    item.meta.is_primary_content && !item.meta.is_id && !item.meta.is_action_link
                } else {
                    !item.meta.is_primary_content && !item.meta.is_id && !item.meta.is_action_link
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .sorted_by(|a, b| {
                a.name.cmp(&b.name)
            })
            .fold(String::new(), |mut acc, item| {
                let trimmed = item.value.trim();
                if acc.is_empty() {
                    acc.push_str(&format!("{}", trimmed));
                } else {
                    acc.push_str(&format!(" • {}", trimmed));
                }
                acc
            });

        if !values.is_empty() {
            if result.is_empty() {
                result.push_str(&values);
            } else {
                let text = format!("{}{}", indentation, values);
                result.push_str(&text);
            }
        }

        for child in &self.children {
            let indentation_factor = if self.values.is_empty() { indentation_factor } else { indentation_factor + 1 };
            child.to_string(filter_secondary_content, indentation_factor, result);
        }
    }
}

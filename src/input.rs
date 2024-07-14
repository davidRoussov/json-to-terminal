use serde::{Serialize, Deserialize};
use std::collections::{HashMap};
use itertools::Itertools;
use std::cmp::Ordering;
use ratatui::{prelude::*, widgets::*};
use textwrap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataMetadata {
    pub is_id: bool,
    pub is_url: bool,
    pub is_page_link: bool,
    pub is_action_link: bool,
    pub is_primary_content: bool,
    pub is_main_primary_content: bool,
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

impl Input {
    pub fn guess_coherent_depth(&self) -> usize {
        let mut primary_content_counts: HashMap<usize, usize> = HashMap::new();

        fn recurse(current_depth: usize, data: &Data, counts: &mut HashMap<usize, usize>) {
            for value in &data.values {
                if value.meta.is_main_primary_content {
                    *counts.entry(current_depth).or_insert(0) += 1;
                }
            }

            for child in &data.children {
                recurse(current_depth + 1, child, counts);
            }
        }

        recurse(0, &self.data, &mut primary_content_counts);

        let total_count: usize = primary_content_counts.values().sum();

        let filtered_counts: HashMap<usize, usize> = primary_content_counts.iter()
            .filter(|&(_, value)| (*value as f64) / (total_count as f64) > 0.1)
            .map(|(&key, &value)| (key, value))
            .collect();

        std::cmp::max(filtered_counts.keys().min().cloned().unwrap() - 1, 0) as usize
    }
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

    pub fn to_lines(
        &self,
        filter_secondary_content: &bool,
        main_content_color: &Color,
        text_color: &Color,
        background_color: &Color,
        result: &mut Vec<Line>
    ) {
        let values: Vec<DataValue> = self.values.iter()
            .filter(|item| {
                if *filter_secondary_content {
                    return item.meta.is_primary_content && !item.meta.is_id && !item.meta.is_action_link;
                }

                !item.meta.is_id && !item.meta.is_action_link
            })
            .collect::<Vec<_>>()
            .into_iter()
            .sorted_by(|a, b| {
                match (a.meta.is_primary_content, b.meta.is_primary_content) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name)
                }
            })
            .cloned()
            .collect();

        let mut lines: Vec<Line> = Vec::new();
        let mut current_line: Line = Line::from(Vec::new());

        for item in values.iter() {
            let value = item.value.trim();
            let fg = if item.meta.is_main_primary_content {
                *main_content_color
            } else {
                *text_color
            };
            let current_line_length: usize = current_line.spans
                .iter()
                .map(|span| span.content.len()).sum();

            if value.len() > 160 {
                if current_line_length > 0 {
                    lines.push(current_line);
                    current_line = Line::from(Vec::new());
                }

                let wrapped = textwrap::wrap(value, &textwrap::Options::new(160));

                for segment in wrapped {
                    lines.push(
                        Line::from(
                            Span::styled(
                                format!("{}", segment),
                                Style::new()
                                    .fg(fg)
                                    .bg(*background_color)
                            )
                        )
                    );
                }
            } else {
                if value.len() + current_line_length > 160 {
                    lines.push(current_line);
                    current_line = Line::from(
                        Span::styled(
                            format!("{}", value),
                            Style::new()
                                .fg(fg)
                                .bg(*background_color)
                        )
                    );
                } else {
                    current_line.spans.push(
                        Span::styled(
                            format!(" {}", value),
                            Style::new()
                                .fg(fg)
                                .bg(*background_color)
                        )
                    );
                }
            }
        }

        let current_line_length: usize = current_line.spans
            .iter()
            .map(|span| span.content.len()).sum();

        if current_line_length > 0 {
            lines.push(current_line);
        }

        result.append(&mut lines);

        for child in &self.children {
            child.to_lines(filter_secondary_content, main_content_color, text_color, background_color, result);
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

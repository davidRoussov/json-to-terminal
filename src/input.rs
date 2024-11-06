use serde::{Serialize, Deserialize};
use std::collections::{HashMap};
use itertools::Itertools;
use std::cmp::Ordering;
use ratatui::{prelude::*, widgets::*};
use textwrap;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContentValueMetadata {
    pub is_title: bool,
    pub is_primary_content: bool,
    pub is_url: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContentValue {
    pub meta: ContentValueMetadata,
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContentMetadataRecursive {
    pub is_root: bool,
    pub parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Content {
    pub id: String,
    #[serde(default)]
    pub values: Vec<ContentValue>,
    #[serde(default)]
    pub inner_content: Vec<Content>,
    #[serde(default)]
    pub children: Vec<Content>,
    #[serde(default)]
    pub lists: Vec<Vec<Content>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub content: Content,
    pub related_content: Content,
}

impl Content {
    pub fn go_down_depth(&self, depth: usize, results: &mut Vec<Content>) {
        if depth == 0 {
            results.push(self.clone());
        } else {
            for child in &self.inner_content {
                child.go_down_depth(depth - 1, results);
            }

            for child in &self.children {
                results.push(child.clone());
            }

            for list in &self.lists {
                for item in list {
                    results.push(item.clone());
                }
            }
        }
    }

    pub fn to_lines(
        &self,
        filter_secondary_content: &bool,
        main_content_color: &Color,
        text_color: &Color,
        background_color: &Color,
        result: &mut Vec<Line>,
        indent_size: usize,
    ) {
        let values: Vec<ContentValue> = self.values.iter()
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

        let indent = " ".repeat(indent_size * 2);

        for (index, item) in values.iter().enumerate() {
            let mut value = item.value.trim();

            let mut fg = if item.meta.is_primary_content {
                *main_content_color
            } else {
                *text_color
            };

            let current_line_length: usize = current_line.spans
                .iter()
                .map(|span| span.content.len()).sum();

            if item.meta.is_url {
                fg = Color::from_str("#0000FF").unwrap();
            }

            let mut bg = *background_color;

            let mut style = Style::new().fg(fg).bg(bg);

            if item.meta.is_url {
                style = style.add_modifier(Modifier::UNDERLINED);
            }

            if  item.meta.is_title {
                style = style.add_modifier(Modifier::BOLD);
            }

            let indent_span = Span::raw(
                format!("{}", indent),
            );

            if value.len() > 160 {
                if current_line_length > 0 {
                    lines.push(current_line);
                    current_line = Line::from(Vec::new());
                }

                let wrapped = textwrap::wrap(value, &textwrap::Options::new(160));

                for segment in wrapped {
                    lines.push(
                        Line::from(vec![
                            indent_span.clone(),
                            Span::styled(
                                format!("{}", segment),
                                style,
                            )
                        ])
                    );
                }
            } else {
                if value.len() + current_line_length > 160 {
                    lines.push(current_line);
                    current_line = Line::from(vec![
                        indent_span.clone(),
                        Span::styled(
                            format!("{}", value),
                            style,
                        )
                    ]);
                } else {
                    current_line.spans.push(indent_span.clone());
                    current_line.spans.push(
                        Span::styled(
                            format!("{}", value),
                            style,
                        )
                    );
                    current_line.spans.push(
                        Span::raw(format!("{}", " ".to_string()))
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

        for child in &self.inner_content {
            child.to_lines(
                filter_secondary_content,
                main_content_color,
                text_color,
                background_color,
                result, 
                indent_size + 1,
            );
        }

        for child in &self.children {
            result.push(Line::from("".to_string()));
            child.to_lines(
                filter_secondary_content,
                main_content_color,
                text_color,
                background_color,
                result,
                indent_size + 2,
            );
        }

        for list in &self.lists {
            result.push(Line::from("---".to_string()));
            for item in list {
                result.push(Line::from("-".to_string()));
                item.to_lines(
                    filter_secondary_content,
                    main_content_color,
                    text_color,
                    background_color,
                    result, 
                    indent_size + 1,
                );
            }
        }
    }
}

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use pulldown_cmark::{html, Options, Parser};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use tokio::fs;

use crate::models::{Post, PostMetadata, Project, ProjectMetadata};

/// Parse frontmatter and content from a markdown file
fn parse_frontmatter<T: serde::de::DeserializeOwned>(content: &str) -> Result<(T, String)> {
    let content = content.trim();

    if !content.starts_with("---") {
        anyhow::bail!("Missing frontmatter");
    }

    let rest = &content[3..];
    let end = rest.find("---").context("Unclosed frontmatter")?;

    let frontmatter = &rest[..end];
    let body = rest[end + 3..].trim();

    let metadata: T = serde_json::from_str(frontmatter)
        .or_else(|_| {
            // Try YAML-like simple parsing for basic cases
            parse_simple_frontmatter(frontmatter)
        })
        .context("Failed to parse frontmatter")?;

    Ok((metadata, body.to_string()))
}

/// Simple frontmatter parser for YAML-like format
fn parse_simple_frontmatter<T: serde::de::DeserializeOwned>(content: &str) -> Result<T> {
    // Convert simple YAML to JSON for parsing
    let mut json_obj = serde_json::Map::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            // Handle arrays (simple case: comma-separated)
            if value.starts_with('[') && value.ends_with(']') {
                let items: Vec<serde_json::Value> = value[1..value.len() - 1]
                    .split(',')
                    .map(|s| serde_json::Value::String(s.trim().trim_matches('"').to_string()))
                    .collect();
                json_obj.insert(key.to_string(), serde_json::Value::Array(items));
            } else if value == "true" {
                json_obj.insert(key.to_string(), serde_json::Value::Bool(true));
            } else if value == "false" {
                json_obj.insert(key.to_string(), serde_json::Value::Bool(false));
            } else {
                json_obj.insert(
                    key.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }
        }
    }

    serde_json::from_value(serde_json::Value::Object(json_obj))
        .context("Failed to parse frontmatter as JSON")
}

/// Convert markdown to HTML with syntax highlighting
pub fn render_markdown(content: &str) -> String {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);

    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();
    let mut events = Vec::new();

    for event in parser {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    _ => String::new(),
                };
                code_content.clear();
            }
            pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                in_code_block = false;

                // Handle Mermaid diagrams specially
                if code_lang == "mermaid" {
                    events.push(pulldown_cmark::Event::Html(
                        format!("<pre class=\"mermaid\">{}</pre>", code_content).into(),
                    ));
                } else {
                    let syntax = ss
                        .find_syntax_by_token(&code_lang)
                        .unwrap_or_else(|| ss.find_syntax_plain_text());

                    let highlighted =
                        highlighted_html_for_string(&code_content, &ss, syntax, theme)
                            .unwrap_or_else(|_| code_content.clone());

                    events.push(pulldown_cmark::Event::Html(
                        format!(
                            "<pre><code class=\"language-{}\">{}</code></pre>",
                            code_lang, highlighted
                        )
                        .into(),
                    ));
                }
            }
            pulldown_cmark::Event::Text(text) if in_code_block => {
                code_content.push_str(&text);
            }
            _ => events.push(event),
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());
    html_output
}

/// Load all posts from a directory
pub async fn load_posts(dir: &Path) -> Result<HashMap<String, Post>> {
    let mut posts = HashMap::new();

    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "md") {
            let content = fs::read_to_string(&path).await?;

            match parse_frontmatter::<PostMetadata>(&content) {
                Ok((metadata, body)) => {
                    let html = render_markdown(&body);
                    let reading_time = Post::estimate_reading_time(&body);

                    let post = Post {
                        metadata: metadata.clone(),
                        content_raw: body,
                        content_html: html,
                        reading_time_minutes: reading_time,
                    };

                    posts.insert(metadata.slug.clone(), post);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse post {:?}: {}", path, e);
                }
            }
        }
    }

    Ok(posts)
}

/// Load all projects from a directory
pub async fn load_projects(dir: &Path) -> Result<HashMap<String, Project>> {
    let mut projects = HashMap::new();

    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "md") {
            let content = fs::read_to_string(&path).await?;

            match parse_frontmatter::<ProjectMetadata>(&content) {
                Ok((metadata, body)) => {
                    let html = render_markdown(&body);

                    let project = Project {
                        metadata: metadata.clone(),
                        content_raw: body,
                        content_html: html,
                    };

                    projects.insert(metadata.slug.clone(), project);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse project {:?}: {}", path, e);
                }
            }
        }
    }

    Ok(projects)
}

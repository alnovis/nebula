use chrono::{DateTime, Utc};
use pulldown_cmark::{Event, Parser};
use serde::{Deserialize, Serialize};

/// Blog post metadata parsed from frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub draft: bool,
    #[serde(default)]
    pub cover_image: Option<String>,
}

/// Complete blog post with content
#[derive(Debug, Clone)]
pub struct Post {
    pub metadata: PostMetadata,
    pub content_raw: String,
    pub content_html: String,
    pub reading_time_minutes: u32,
}

impl Post {
    /// Estimate reading time based on word count (200 words per minute).
    /// Raw HTML, such as inline SVG figures, is not counted.
    pub fn estimate_reading_time(content: &str) -> u32 {
        let word_count: usize = Parser::new(content)
            .map(|event| match event {
                Event::Text(text) | Event::Code(text) => text.split_whitespace().count(),
                _ => 0,
            })
            .sum();
        ((word_count as f64) / 200.0).ceil() as u32
    }
}

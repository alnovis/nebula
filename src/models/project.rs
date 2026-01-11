use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Project metadata parsed from frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub status: ProjectStatus,
    pub github_url: Option<String>,
    pub demo_url: Option<String>,
    pub featured: bool,
}

/// Project development status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Completed,
    Archived,
    Planned,
}

/// Complete project with content
#[derive(Debug, Clone)]
pub struct Project {
    pub metadata: ProjectMetadata,
    pub content_raw: String,
    pub content_html: String,
}

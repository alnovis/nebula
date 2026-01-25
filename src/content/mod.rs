pub mod markdown;

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use tracing::info;

use crate::models::{Post, Project};

/// In-memory store for all content
#[derive(Debug, Default)]
pub struct ContentStore {
    pub posts: HashMap<String, Post>,
    pub projects: HashMap<String, Project>,
}

impl ContentStore {
    /// Load all content from the filesystem
    pub async fn load(content_dir: &str) -> Result<Self> {
        let mut store = Self::default();

        let blog_dir = Path::new(content_dir).join("blog");
        if blog_dir.exists() {
            store.posts = markdown::load_posts(&blog_dir).await?;
            info!("Loaded {} blog posts", store.posts.len());
        }

        let projects_dir = Path::new(content_dir).join("projects");
        if projects_dir.exists() {
            store.projects = markdown::load_projects(&projects_dir).await?;
            info!("Loaded {} projects", store.projects.len());
        }

        Ok(store)
    }

    /// Get all published posts sorted by date (newest first)
    pub fn published_posts(&self) -> Vec<&Post> {
        let mut posts: Vec<_> = self.posts.values().filter(|p| !p.metadata.draft).collect();
        posts.sort_by(|a, b| b.metadata.date.cmp(&a.metadata.date));
        posts
    }

    /// Get all projects sorted by date (newest first)
    pub fn all_projects(&self) -> Vec<&Project> {
        let mut projects: Vec<_> = self.projects.values().collect();
        projects.sort_by(|a, b| b.metadata.date.cmp(&a.metadata.date));
        projects
    }

    /// Get featured projects
    pub fn featured_projects(&self) -> Vec<&Project> {
        self.all_projects()
            .into_iter()
            .filter(|p| p.metadata.featured)
            .collect()
    }
}

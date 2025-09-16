
use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub url: Option<String>,
    pub body: Option<String>,
    pub short_description: Option<String>,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, user_id: Uuid, title: &str, short_description: &str, url: &Option<String>, body: &Option<String>) -> anyhow::Result<Post>;
    async fn list_new(&self, after: Option<(DateTime<Utc>, Uuid)>, limit: i64) -> anyhow::Result<Vec<Post>>;
    async fn list_top(&self, after: Option<(DateTime<Utc>, Uuid)>, limit: i64) -> anyhow::Result<Vec<Post>>;
    async fn update(&self, post_id: Uuid, title: &str, short_description: &str, url: &Option<String>, body: &Option<String>) -> anyhow::Result<Post>;
}

use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            created_at: user.created_at,
        }
    }
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, email: &str, username: &str, password_hash: &str) -> anyhow::Result<User>;
    async fn find_by_email_or_username(&self, key: &str) -> anyhow::Result<Option<User>>;
    async fn verify_token(&self, token: &str) -> anyhow::Result<Uuid>;
}
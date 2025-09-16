
#[derive(Debug, Clone)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait CommentRepository: Send + Sync {
    async fn create(&self, post_id: Uuid, user_id: Uuid, parent_id: Option<Uuid>, body: &str) -> anyhow::Result<Comment>;
    async fn list_for_post(&self, post_id: Uuid) -> anyhow::Result<Vec<Comment>>;
}
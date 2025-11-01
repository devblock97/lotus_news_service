use uuid::Uuid;


#[derive(Debug)]
pub struct Vote {
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub value: i16,
}

#[async_trait::async_trait]
pub trait VoteRepository: Send + Sync {
    async fn vote(&self, vote: Vote) -> Result<(), anyhow::Error>;
    async fn get_score(&self, post_id: Uuid) -> Result<i16, anyhow::Error>;
}
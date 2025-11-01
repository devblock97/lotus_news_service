use std::sync::Arc;

use uuid::Uuid;

use crate::domain::votes::{Vote, VoteRepository};


pub struct VoteService {
    pub repo: Arc<dyn VoteRepository>,
}

impl VoteService {
    pub fn new(repo: Arc<dyn VoteRepository>) -> Self { Self { repo } }

    pub async fn vote(&self, user_id: Uuid, post_id: Uuid, value: i16) -> Result<(), anyhow::Error> {
        if value != 1 && value != -1 {
            return Err(anyhow::anyhow!("Vote must be +1 or - 1"));
        }
        let vote = Vote { user_id, post_id, value };
        self.repo.vote(vote).await
    }

    pub async fn get_score(&self, post_id: Uuid) -> Result<i16, anyhow::Error> {
        self.repo.get_score(post_id).await
    }
}
use anyhow::Ok;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::votes::{Vote, VoteRepository};


pub struct PgVoteRepository {
    pub pool: PgPool,
}

#[async_trait::async_trait]
impl VoteRepository for PgVoteRepository {
    async fn vote(&self, vote: Vote) -> Result<(), anyhow::Error> {
        let existing = sqlx::query!(
            r#"
            SELECT value FROM votes 
            WHERE user_id = $1 AND post_id = $2
            "#,
            vote.user_id,
            vote.post_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = existing {
            if record.value == vote.value {
                sqlx::query!(
                    "DELETE FROM votes WHERE user_id = $1 AND post_id = $2",
                    vote.user_id,
                    vote.post_id
                )
                .execute(&self.pool)
                .await?;
            } else {
                sqlx::query!(
                    "UPDATE votes SET value = $3 WHERE user_id = $1 AND post_id = $2",
                    vote.user_id,
                    vote.post_id,
                    vote.value
                )
                .execute(&self.pool)
                .await?;
            }
        } else {
            sqlx::query!(
                "INSERT INTO votes (user_id, post_id, value) VALUES ($1, $2, $3)",
                vote.user_id,
                vote.post_id,
                vote.value
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn get_score(&self, post_id: Uuid) -> Result<i16, anyhow::Error> {
        let record = sqlx::query!(
            r#"SELECT COALESCE(SUM(value), 0) as score
               FROM votes
               WHERE post_id = $1
            "#,
            post_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(record.score.unwrap_or(0) as i16)
    }


}
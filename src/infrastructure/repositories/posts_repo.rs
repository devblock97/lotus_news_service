use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::{Pool, Postgres};

use crate::domain::posts::{Post, PostRepository};

type DbPool = Pool<Postgres>;


pub struct PgPostRepository { pub pool: DbPool }

#[async_trait]
impl PostRepository for PgPostRepository {
    async fn create(&self, user_id: Uuid, title: &str, short_description: &str, url: &Option<String>, body: &Option<String>) -> anyhow::Result<Post> {
        let id = Uuid::new_v4();
        let rec = sqlx::query_as!(PostRow,
            r#"INSERT INTO posts (id, user_id, title, short_description, url, body)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id, user_id, title, short_description, url, body, score, created_at
            "#,
            id, user_id, title, short_description, url.as_deref(), body.as_deref()
        )
        .fetch_one(&self.pool).await?;

        Ok(rec.into())
    }

    async fn list_new(&self, after: Option<(DateTime<Utc>, Uuid)>, limit: i64) -> anyhow::Result<Vec<Post>> {
        let rows = if let Some((created_at, id)) = after {
            sqlx::query_as!(PostRow,
                r#"SELECT id, user_id, title, short_description, url, body, score, created_at FROM posts
                    WHERE (created_at, id) < ($1, $2)
                    ORDER BY created_at DESC, id DESC LIMIT $3
                "#,
                created_at, id, limit
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(PostRow,
                r#"SELECT id, user_id, title, short_description, url, body, score, created_at FROM posts
                    ORDER BY created_at DESC, id DESC LIMIT $1
                "#,
                limit
            ).fetch_all(&self.pool).await?
        };
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_top(&self, after: Option<(DateTime<Utc>, Uuid)>, limit: i64) -> anyhow::Result<Vec<Post>> {
        let rows = if let Some((created_at, id)) = after {
            sqlx::query_as!(PostRow,
                r#"SELECT id, user_id, title, short_description, url, body, score, created_at FROM posts
                    WHERE (created_at, id) < ($1, $2)
                    ORDER BY score DESC, created_at DESC, id DESC LIMIT $3
                "#,
                created_at, id, limit
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(PostRow,
                r#"SELECT id, user_id, title, short_description, url, body, score, created_at FROM posts
                    ORDER BY score DESC, created_at DESC, id DESC LIMIT $1
                "#,
                limit
            ).fetch_all(&self.pool).await?
        };
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&self, post_id: Uuid, title: &str, short_description: &str, url: &Option<String>, body: &Option<String>) -> anyhow::Result<Post> {
        let rec = sqlx::query_as!(PostRow, 
            r#"UPDATE posts
                SET title = $1, short_description = $2, url = $3, body = $4
                WHERE id = $5
                RETURNING id, user_id, title, short_description, url, body, score, created_at
            "#,
            title, short_description, url.as_deref(), body.as_deref(), post_id
        )
        .fetch_one(&self.pool).await?;

        Ok(rec.into())
    }
}

#[derive(sqlx::FromRow)]
struct PostRow {
    id: Uuid,
    user_id: Uuid,
    title: String,
    short_description: Option<String>,
    url: Option<String>,
    body: Option<String>,
    score: i32,
    created_at: DateTime<Utc>,
}

impl From<PostRow> for Post {
    fn from(r: PostRow) -> Self {
        Self { 
            id: r.id, 
            user_id: r.user_id, 
            title: r.title, 
            short_description: r.short_description,
            url: r.url, 
            body: r.body, 
            score: r.score, 
            created_at: r.created_at
        }
    }
}
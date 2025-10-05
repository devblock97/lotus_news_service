use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::{Pool, Postgres};
use crate::infrastructure::db::DbPool;

use crate::domain::posts::{Post, PostRepository};

pub struct PgPostRepository { pub pool: DbPool }

#[async_trait]
impl PostRepository for PgPostRepository {
    async fn create(&self, user_id: Uuid, title: &str, short_description: &str, url: &Option<String>, body: &Option<String>) -> anyhow::Result<Post> {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO posts (id, user_id, title, short_description, url, body)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            id, user_id, title, short_description, url.as_deref(), body.as_deref()
        )
        .execute(&self.pool).await?;

        let post = sqlx::query_as!(
            Post,
            r#"SELECT p.id, p.user_id, p.title, p.url, p.body, p.short_description, p.score, p.created_at, u.avatar, u.username as author_username
                FROM posts p
                JOIN users u ON p.user_id = u.id
                WHERE p.id = $1
            "#,
            id
        )
        .fetch_one(&self.pool).await?;

        Ok(post)
    }

    async fn list_new(&self, after: Option<(DateTime<Utc>, Uuid)>, limit: i64) -> anyhow::Result<Vec<Post>> {
        let posts = if let Some((created_at, id)) = after {
            sqlx::query_as!(
                Post,
                r#"SELECT p.id, p.user_id, p.title, p.url, p.body, p.short_description, p.score, p.created_at, u.avatar, u.username as author_username
                    FROM posts p
                    JOIN users u ON p.user_id = u.id
                    WHERE (p.created_at, p.id) < ($1, $2)
                    ORDER BY p.created_at DESC, p.id DESC LIMIT $3
                "#,
                created_at, id, limit
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(
                Post,
                r#"SELECT p.id, p.user_id, p.title, p.url, p.body, p.short_description, p.score, p.created_at, u.avatar, u.username as author_username
                    FROM posts p
                    JOIN users u ON p.user_id = u.id
                    ORDER BY p.created_at DESC, p.id DESC LIMIT $1
                "#,
                limit
            ).fetch_all(&self.pool).await?
        };
        Ok(posts)
    }

    async fn list_top(&self, after: Option<(DateTime<Utc>, Uuid)>, limit: i64) -> anyhow::Result<Vec<Post>> {
        let posts = if let Some((created_at, id)) = after {
            sqlx::query_as!(
                Post,
                r#"SELECT p.id, p.user_id, p.title, p.url, p.body, p.short_description, p.score, p.created_at, u.avatar, u.username as author_username
                    FROM posts p
                    JOIN users u ON p.user_id = u.id
                    WHERE (p.created_at, p.id) < ($1, $2)
                    ORDER BY p.score DESC, p.created_at DESC, p.id DESC LIMIT $3
                "#,
                created_at, id, limit
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(
                Post,
                r#"SELECT p.id, p.user_id, p.title, p.url, p.body, p.short_description, p.score, p.created_at, u.avatar, u.username as author_username
                    FROM posts p
                    JOIN users u ON p.user_id = u.id
                    ORDER BY p.score DESC, p.created_at DESC, p.id DESC LIMIT $1
                "#,
                limit
            ).fetch_all(&self.pool).await?
        };
        Ok(posts)
    }

    async fn update(&self, post_id: Uuid, title: &str, short_description: &str, url: &Option<String>, body: &Option<String>) -> anyhow::Result<Post> {
        let post = sqlx::query_as!(
            Post,
            r#"
                UPDATE posts p
                SET title = $1, short_description = $2, url = $3, body = $4
                FROM users u
                WHERE p.id = $5 AND p.user_id = u.id -- Thêm u.id vào WHERE nếu cần JOIN
                RETURNING p.id, p.user_id, p.title, p.url, p.body, p.short_description, p.score, p.created_at, u.avatar, u.username as author_username
            "#,
            title,
            short_description,
            url.as_deref(),
            body.as_deref(),
            post_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    async fn search_by_title(&self, query: &str, after: Option<(DateTime<Utc>, Uuid)>, limit: i64) -> anyhow::Result<Vec<Post>> {
        let posts = if let Some((created_at, id)) = after {
            sqlx::query_as!(
                Post,
                r#"SELECT p.id, p.user_id, p.title, p.url, p.body, p.short_description, p.score, p.created_at, u.avatar, u.username as author_username
                    FROM posts p
                    JOIN users u ON p.user_id = u.id
                    WHERE p.title ILIKE $1 AND (p.created_at, p.id) < ($2, $3)
                    ORDER BY p.created_at DESC, p.id DESC LIMIT $4
                "#,
                format!("%{}%", query), created_at, id, limit
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(
                Post,
                r#"SELECT p.id, p.user_id, p.title, p.url, p.body, p.short_description, p.score, p.created_at, u.avatar, u.username as author_username
                    FROM posts p
                    JOIN users u ON p.user_id = u.id
                    WHERE p.title ILIKE $1
                    ORDER BY p.created_at DESC, p.id DESC LIMIT $2
                "#,
                format!("%{}%", query), limit
            ).fetch_all(&self.pool).await?
        };
        Ok(posts)
    }
}
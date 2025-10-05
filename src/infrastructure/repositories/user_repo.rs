use uuid::Uuid;
use async_trait::async_trait;

use crate::{domain::users::{User, UserRepository}, infrastructure::{auth, db::DbPool}};

pub struct PgUserRepository {
    pub pool: DbPool,
    pub jwt: auth::JwtKeys,
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, email: &str, username: &str, avatar: &str, password_hash: &str) -> anyhow::Result<User> {
        let id = Uuid::new_v4();
        let rec = sqlx::query_as!(UserRow, 
            r#"INSERT INTO users (id, email, username, avatar, password_hash)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, email, username, avatar, password_hash, created_at"#,
            id, email, username, avatar, password_hash
        )
        .fetch_one(&self.pool).await?;

        Ok(rec.into())
    }
    
    async fn find_by_email_or_username(&self, key: &str) -> anyhow::Result<Option<User>> {
        let row = sqlx::query_as!(UserRow,
            r#"SELECT id, email, username, avatar, password_hash, created_at 
                FROM users
                WHERE email = $1 OR username = $1 LIMIT 1"#, key
        ).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| r.into()))
    }

    async fn verify_token(&self, token: &str) -> anyhow::Result<Uuid> {
        let claims = self.jwt.verify(token)?;
        Ok(claims)
    }

}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    username: String,
    avatar: String,
    password_hash: String,
    created_at: chrono::DateTime<chrono::Utc>
}

impl From<UserRow> for User  {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            email: value.email,
            username: value.username,
            avatar: value.avatar,
            password_hash: value.password_hash,
            created_at: value.created_at,
        }
    }
}
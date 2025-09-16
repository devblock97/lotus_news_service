use std::sync::Arc;

use uuid::Uuid;
use validator::Validate;
use crate::infrastructure::auth;
use crate::{application::error::AppError, domain::users::{User, UserRepository}};

#[derive(Debug, Validate)]
pub struct SignupInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 40))]
    pub username: String,
    #[validate(length(min = 6, max = 128))]
    pub password: String,
}

pub struct UserService {
    pub repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self { Self { repo } }

    /// Registers a user after validation & uniqueness checks.
    pub async fn signup(&self, username: String, email: String, password: String) -> Result<User, AppError> {
        // input.validate().map_err(|e| AppError::validation(e.to_string()))?;

        // normalize
        let email = email.to_lowercase();
        let username = username.trim();

        // uniqueness
        if let Some(_) = self.repo.find_by_email_or_username(&email).await? {
            return Err(AppError::conflict("email already registerd"));
        }

        if let Some(_) = self.repo.find_by_email_or_username(username).await? {
            return Err(AppError::conflict("username ready taken"));
        }

        let hash = auth::hash_password(&password)?;
        let user = self.repo.create(&email, username, &hash).await?;
        Ok(user)
    }

    /// Returns the user if credentials are valid.
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<User, AppError> {
        let key = email.to_lowercase();
        let Some(user) = self.repo.find_by_email_or_username(&key).await? else {
            return Err(AppError::Unauthorized)
        };

        let ok = auth::verify_password(password, &user.password_hash)?;
        if !ok { return Err(AppError::Unauthorized)}
        Ok(user)
    }

    pub async fn verify_token(&self, token: &str) -> Result<Option<Uuid>, anyhow::Error> {
        let user_id = self.repo.verify_token(token).await?;
        Ok(Some(user_id))
    }
}
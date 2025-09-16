use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::Deserialize;
use validator::Validate;

use crate::domain::posts::{Post, PostRepository};
use crate::application::error::AppError;
use crate::application::utils::{validation, profanity};

#[derive(Debug, Validate, Deserialize)]
pub struct CreatePostInput {
    #[validate(length(min = 3, max = 300))]
    pub title: String,
    pub short_description: String,
    pub url: Option<String>,
    pub body: Option<String>
}

pub struct PostService {
    repo: Arc<dyn PostRepository>,
}

impl PostService {
    pub fn new(repo: Arc<dyn PostRepository>) -> Self { Self { repo }}

    pub async fn create(&self, user_id: Uuid, input: CreatePostInput) -> Result<Post, AppError> {
        input.validate().map_err(|e| AppError::validation(e.to_string()))?;

        let has_url = input.url.as_ref().is_some();
        let has_body = input.body.as_ref().is_some();

        if !has_url && !has_body {
            return Err(AppError::validation("Either url or body must be provided".to_string()));
        }

        if !has_url && input.body.as_ref().unwrap().trim().is_empty() {
            return Err(AppError::validation("Body cannot be empty if url is not provided".to_string()));
        }

        validation::validate_http_url(&input.url)
            .map_err(|e| AppError::validation(e.to_string()))?;

        if profanity::contains_profanity(&input.title) {
            return Err(AppError::validation("Title contains inappropriate language".to_string()));
        }

        if let Some(body) = &input.body {
            if profanity::contains_profanity(body) {
                return Err(AppError::validation("Body contains inappropriate language".to_string()));
            }
        }

        let post = self.repo.create(user_id, &input.title, &input.short_description,&input.url, &input.body).await?;

        Ok(post)
    }

    // pub async fn update(&self, post_id: Uuid, title: &str, short_description: &str, url: &Option<String>, body: &Option<String>) -> Result<Post, AppError> {
    //     if title.len() < 3 || title.len() > 300 {
    //         return Err(AppError::validation("Title must be between 3 and 300 characters".to_string()));
    //     }

    //     let has_url = url.as_ref().is_some();
    //     let has_body = body.as_ref().is_some();

    //     if !has_url && !has_body {
    //         return Err(AppError::validation("Either url or body must be provided".to_string()));
    //     }

    //     if !has_url && body.as_ref().unwrap().trim().is_empty() {
    //         return Err(AppError::validation("Body cannot be empty if url is not provided".to_string()));
    //     }

    //     validation::validate_http_url(url)
    //         .map_err(|e| AppError::validation(e.to_string()))?;

    //     if profanity::contains_profanity(title) {
    //         return Err(AppError::validation("Title contains inappropriate language".to_string()));
    //     }

    //     if let Some(body) = body {
    //         if profanity::contains_profanity(body) {
    //             return Err(AppError::validation("Body contains inappropriate language".to_string()));
    //         }
    //     }

    //     let post = self.repo.update(post_id, title, short_description, url, body).await?;

    //     Ok(post)
    // }

    pub async fn update(&self, post_id: Uuid, title: &str, short_description: &str, url: &Option<String>, body: &Option<String>) -> Result<Post, AppError> {
        if title.len() < 3 || title.len() > 300 {
            return Err(AppError::validation("Title must be between 3 and 300 characters".to_string()));
        }

        let has_url = url.as_ref().is_some();
        let has_body = body.as_ref().is_some();

        if !has_url && !has_body {
            return Err(AppError::validation("Either url or body must be provided".to_string()));
        }

        let post = self.repo.update(post_id, title, short_description, url, body).await?;
        Ok(post)
    }

    pub async fn list_new(
        &self,
        after: Option<(DateTime<Utc>, Uuid)>,
        limit: i64
    ) -> Result<Vec<Post>, AppError> {
        Ok(self.repo.list_new(after, limit).await?)
    }

    pub async fn list_top(
        &self,
        after: Option<(DateTime<Utc>, Uuid)>,
        limit: i64
    ) -> Result<Vec<Post>, AppError> {
        Ok(self.repo.list_top(after, limit).await?)
    }
}
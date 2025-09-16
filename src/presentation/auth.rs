use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};

use axum_extra::TypedHeader;
use headers::Authorization;
use headers::authorization::Bearer;

use uuid::Uuid;

use crate::application::user_service::UserService;

pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    Arc<UserService>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization: Bearer <token>
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing token".into()))?;

        // Get UserService from state
        let user_service: Arc<UserService> = FromRef::from_ref(state);

        // Verify token -> extract user_id
        let user_id = user_service
            .verify_token(bearer.token())
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

        Ok(AuthUser { user_id })
    }
}

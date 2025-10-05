use axum::{Json, http::StatusCode, extract::State};
use serde::Deserialize;
use crate::presentation::ApiState;
use crate::domain::users::UserPublic;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn signup(
    State(state): State<ApiState>,
    Json(payload): Json<SignupRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.user_service.signup(payload.username, payload.email, payload.avatar, payload.password)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

pub async fn login(
    State(state): State<ApiState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = state.user_service.authenticate(&payload.email, &payload.password).await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let token = state.jwt_keys.issue(user.id, 7)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to issue token".to_string()))?;

    let user_public: UserPublic = user.into();

    Ok(Json(serde_json::json!({ 
        "token": token, 
        "user": user_public,
    })))
}

use axum::{ Json, extract::{Path, State} };
use serde::Deserialize;
use uuid::Uuid;
use crate::presentation::{auth::AuthUser, ApiState};
pub struct VoteRequest {
    pub value: i16,
}

pub async fn vote_on_post<R>(
    State(state): State<ApiState>,
    AuthUser { user_id }: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(payload): Json<VoteRequest>,
) -> Result<Json<&'static str>, (axum::http::StatusCode, String)>
{
    state.vote_service
        .vote(user_id, post_id, payload.value)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json("Vote recorded"))
}
use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::application::posts_service::CreatePostInput;
use crate::domain::posts::Post;
use crate::presentation::{auth::AuthUser, ApiState};


#[derive(Deserialize)]
pub struct ListPostQuery {
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[axum::debug_handler]
pub async fn create_post(
    State(state): State<ApiState>,
    AuthUser { user_id }: AuthUser,
    Json(_payload): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<Post>), (StatusCode, String)> {
    let post = state.post_service.create(user_id, _payload)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(post)))
}

#[axum::debug_handler]
pub async fn update_post(
    State(state): State<ApiState>, 
    Path(post_id): Path<Uuid>,
    Json(_payload): Json<CreatePostInput>
) -> Result<(StatusCode, Json<Post>), (StatusCode, String)> {
    let post = state.post_service.update(post_id, &_payload.title, &_payload.short_description, &_payload.url, &_payload.body)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    Ok((StatusCode::OK, Json(post)))
}

pub async fn list_posts(
    State(state): State<ApiState>,
    Query(query): Query<ListPostQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20);
    if limit <= 0 {
        return Err((StatusCode::BAD_REQUEST, "Limit must be greater than 0".to_string()));
    }

    let posts = state.post_service.list_new(None, limit).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(serde_json::json!({ "posts": posts })))
}


use axum::{
    extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade}, http::StatusCode, response::IntoResponse, Json
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

    // Send the enw post to all WebSocket listeners
    // We ignore the result, as it's okay if there are no active listeners
    let _ = state.post_broadcaster.send(post.clone());

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

#[axum::debug_handler]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket,state: ApiState,) {
    let mut rx = state.post_broadcaster.subscribe();

    loop {
        tokio::select! {
            // Receive a new post from the broadcast channel
            Ok(post) = rx.recv() => {
                // Serialize the post to JSON and send it to the client
                if socket.send(serde_json::to_string(&post).unwrap().into()).await.is_err() {
                    break;
                }
            }
            // Receive a message from the client (optional, but good for health checks)
            Some(Ok(msg)) = socket.recv() => {
                if let axum::extract::ws::Message::Close(_) = msg {
                    // Client sent a close message
                    break;
                }
            }
            else => {
                // Client disconnected
                break;
            }
        }
    }
}

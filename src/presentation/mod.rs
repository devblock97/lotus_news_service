use std::sync::Arc;
use axum::routing::{delete, post, put};
use axum::{routing::get, Router};
use axum::extract::FromRef;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::domain::posts::Post;
use crate::infrastructure::repositories::user_repo::PgUserRepository;
use crate::AppContext;
use crate::application::posts_service::PostService;
use crate::application::user_service::UserService;
use crate::infrastructure::repositories::posts_repo::PgPostRepository;


mod auth;
mod post_handler;
mod user_handler;

use crate::infrastructure::auth::JwtKeys;

#[derive(Clone, FromRef)]
pub struct ApiState {
    user_service: Arc<UserService>,
    post_service: Arc<PostService>,
    jwt_keys: Arc<JwtKeys>,
    post_broadcaster: broadcast::Sender<Post>,
}

pub fn routes(ctx: AppContext) -> Router {
    let posts_repo: Arc<dyn crate::domain::posts::PostRepository> = Arc::new(PgPostRepository { pool: ctx.pool.clone() });
    let post_service = Arc::new(PostService::new(posts_repo));
    
    let jwt_keys = Arc::new(JwtKeys::new(&ctx.jwt_secret));
    let user_repo: Arc<dyn crate::domain::users::UserRepository> = Arc::new(PgUserRepository { pool: ctx.pool.clone(), jwt: (*jwt_keys).clone() });
    let user_service = Arc::new(UserService::new(user_repo));

    let (tx, _) = broadcast::channel(100);

    let state = ApiState {
        user_service,
        post_service,
        jwt_keys,
        post_broadcaster: tx,
    };

    Router::new()
        .route("/signup", post(user_handler::signup))
        .route("/login", post(user_handler::login))
        .route("/posts", get(post_handler::list_posts))
        .route("/posts", post(post_handler::create_post))
        .route("/posts/{id}", delete(post_handler::delete_post))
        .route("/posts/{id}", put(post_handler::update_post))
        .route("/ws/posts", get(post_handler::ws_handler))
        .with_state(state)
}

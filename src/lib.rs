pub mod config;
pub mod app;
pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod presentation;

use crate::app::build_router;
use axum::Router;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppContext {
pub pool: Pool<Postgres>,
    pub jwt_secret: String,
}

pub async fn build_app(ctx: AppContext) -> Router {
    build_router(ctx).await
}
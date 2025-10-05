use axum::{routing::get, Router};

use crate::{infrastructure::observability, presentation, AppContext};

pub async fn build_router(ctx: AppContext) -> Router {
    let (set_request_id, propagate_request_id, trace) = observability::middleware();

    let api = presentation::routes(ctx.clone());

    Router::new()
        .route("/health", get(|| async {"OK"}))
        .nest("/api", api)
        .layer(observability::cors_layer())
        .layer(set_request_id)
        .layer(propagate_request_id)
        .layer(trace)
}
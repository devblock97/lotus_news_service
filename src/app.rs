use axum::{routing::get, Router};
use tower_http::{cors::{Any, CorsLayer}, request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer}, trace::TraceLayer};

use crate::presentation;
use crate::AppContext;


pub async fn build_router(ctx: AppContext) -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let make_req_id = MakeRequestUuid::default();
    
    let api = presentation::routes(ctx.clone());

    Router::new()
        .route("/health", get(|| async {"OK"}))
        .nest("/api", api)
        .layer(cors)
        .layer(SetRequestIdLayer::x_request_id(make_req_id))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
}
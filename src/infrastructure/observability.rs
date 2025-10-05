use tower_http::{classify::{ServerErrorsAsFailures, SharedClassifier}, cors::{Any, CorsLayer}, request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer}, trace::TraceLayer};

/// Convenience: CORS layer used by app.
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)
}

/// Telemetry layers: request id + trace
pub fn middleware() -> (SetRequestIdLayer<MakeRequestUuid>, PropagateRequestIdLayer, TraceLayer<SharedClassifier<ServerErrorsAsFailures>>) {
    let make_id = MakeRequestUuid::default();
    (SetRequestIdLayer::x_request_id(make_id), PropagateRequestIdLayer::x_request_id(), TraceLayer::new_for_http())
}

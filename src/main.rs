use postgres::config;
use lotus_news_service::{build_app, infrastructure::db, AppContext};
use lotus_news_service::config::Config;

use axum::{extract::{FromRequestParts, Path, State}, http::{header, request::Parts, StatusCode}, Json};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use url::Url;
use uuid::Uuid;
use chrono::{DateTime, Duration, Utc};
use validator::{Validate};
use tracing_subscriber::{self, prelude::*};
use bcrypt;
use axum::extract::FromRef;
use tracing::error;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    jwt_encoding: EncodingKey,
    #[allow(dead_code)] // Will be used for JWT token validation in auth middleware
    jwt_decoding: DecodingKey,
}

#[derive(Debug, Deserialize, Validate)]
struct SignupPayload {
    #[validate(email(message = "invalid email"))]
    email: String,

    #[validate(length(min = 6, message = "password too short (min 6)"))]
    password: String,

    #[validate(length(min = 3, max = 20, message = "username length 3..20"))]
    username: String,
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    email_or_username: String,
    password: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct UserPublic {
    id: Uuid,
    email: String,
    username: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // subject: user id
    sub: String,
    exp: usize,
}

// AuthUser struct definition
#[derive(Debug)]
struct AuthUser {
    user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ApiError>);
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) else {
            return Err(http_err(StatusCode::UNAUTHORIZED, "missing Authorization header"))
        };

        let auth_str = auth_header.to_str().unwrap_or_default();
        if !auth_str.starts_with("Bearer ") {
            return Err(http_err(StatusCode::UNAUTHORIZED, "invalid auth scheme"))
        }
        let token = &auth_str[7..];

        // Parse user_id from claims
        let validation = Validation::new(Algorithm::HS256);
        let data = decode::<Claims>(token, &state.jwt_decoding, &validation)
            .map_err(|_| http_err(StatusCode::UNAUTHORIZED, "invalid or expired token"))?;
        let user_id = Uuid::parse_str(&data.claims.sub)
            .map_err(|_| http_err(StatusCode::UNAUTHORIZED, "invalid token subject"))?;

        Ok(AuthUser { user_id })
    }
}


// Posts
#[derive(Debug, Deserialize, Validate)]
struct CreatePostPayload {
    #[validate(length(min = 3, max = 300, message = "title length 3..300"))]
    title: String,
    #[validate(custom(function = "validate_optional_url"))]
    url: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct PostPublic {
    id: Uuid,
    user_id: Uuid,
    title: String,
    url: Option<String>,
    body: Option<String>,
    score: i32,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct VotePayload {
    value: i16, // must be 1 or -1
}

#[derive(Debug, Deserialize, Validate)]
struct CreateCommentPayload {
    parent_id: Option<Uuid>,
    #[validate(length(min = 1, max = 10_000))]
    body: String
}

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
struct CommentRow {
    id: Uuid,
    post_id: Uuid,
    user_id: Uuid,
    parent_id: Option<Uuid>,
    body: String,
    created_at: DateTime<Utc>,
}

// Nested comment node we return
#[derive(Debug, Serialize)]
struct CommentNode {
    id: Uuid,
    post_id: Uuid,
    user_id: Uuid,
    parent_id: Option<Uuid>,
    body: String,
    created_at: DateTime<Utc>,
    children: Vec<CommentNode>,
}

// Cursor for pagination: (created_at, id)
#[derive(Debug, Serialize, Deserialize)]
struct Cursor {
    created_at: DateTime<Utc>,
    id: Uuid,
}

#[derive(Debug, Serialize)]
struct PaginatedPosts {
    posts: Vec<PostPublic>,
    next_cursor: Option<Cursor>
}

// ---------------------------- Validation helpers ----------------------------
fn validate_optional_url(v: &str) -> Result<(), validator::ValidationError> {
    // empty string treated as None by clients sometimes
    if v.trim().is_empty() { return Ok(()); }

    if Url::parse(v).is_err() {
        let mut e = validator::ValidationError::new("invalid_url");
        e.message = Some("invalid URL".into());
        return Err(e);
    }

    Ok(())
}

// toy profanity filter (extend as needed)
fn censor(s: &str) -> String {
    // lower-effort example list
    const BAD: &[&str] = &["damn", "hell"];
    let mut out = s.to_string();
    for &w in BAD {
        let mask = "*".repeat(w.len());
        out = out.replace(w, &mask).replace(&w.to_ascii_uppercase(), &mask);
    }
    out
}

// ---------------------------- Error helper ----------------------------------
fn http_err(status: StatusCode, msg: &str) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.to_string() }))
}

// Helper to convert validation errors to HTTP errors
fn validation_to_http_err(errors: validator::ValidationErrors) -> (StatusCode, Json<ApiError>) {
    let error_msg = errors
        .field_errors()
        .iter()
        .map(|(field, errors)| {
            let messages: Vec<String> = errors.iter().filter_map(|e| e.message.as_ref().map(|msg| msg.to_string())).collect();
            format!("{}: {}", field, messages.join(", "))
        })
        .collect::<Vec<_>>()
        .join("; ");
    
    http_err(StatusCode::BAD_REQUEST, &error_msg)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // ------------------------------
    // 1. Setup tracing / logging
    // ------------------------------
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(
            |_| tracing_subscriber::EnvFilter::new("info,sqlx=warn,tower_http_info"),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    // let pool = PgPoolOptions::new()
    //     .max_connections(10)
    //     .connect(&database_url)
    //     .await?;

    // ------------------------------
    // 2. Load configuration
    // ------------------------------
    let cfg = Config::from_env();

    // ------------------------------
    // 3. Setup DB connection
    // ------------------------------
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    // Optional: run migrations in-process (simple files loader)
    // db::apply_sql_folder(&pool, "migrations").await?;

    let ctx = AppContext { pool, jwt_secret: cfg.jwt_secret.clone() };
    let app = build_app(ctx).await;

    tracing::info!("listening on {}", cfg.bind_addr);
    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    axum::serve(listener, app).await.unwrap();

    Ok(())

    // JWT secret key - require it to be set in environment for security
    // let jwt_secret = std::env::var("JWT_SECRET")
    //     .expect("JWT_SECRET must be set in environment variables");
    
    // let jwt_encoding = EncodingKey::from_secret(jwt_secret.as_ref());
    // let jwt_decoding = DecodingKey::from_secret(jwt_secret.as_ref());

    // let state = AppState { 
    //     db: pool,
    //     jwt_encoding,
    //     jwt_decoding,
    // };

    // // More restrictive CORS in production
    // let cors = CorsLayer::new()
    //     .allow_origin(Any) // TODO: Replace with specific origins in production
    //     .allow_methods(Any)
    //     .allow_headers(Any);

    // // Prometheus metrics
    // let (prom_layer, _metric_handle) = PrometheusMetricLayer::pair();

    // // Request IDs + tracing
    // let make_req_id = MakeRequestUuid::default();

    // let app = Router::new()
    //     .route("/health", get(|| async { "OK" }))
    //     .route("/signup", post(signup))
    //     .route("/login", post(login))

    //     .route("/posts", get(list_posts))
    //     .route("/posts", post(create_post))

    //     .route("/posts/{id}/comments", post(create_comment))
    //     .route("/posts/{id}/comments", get(list_comments))
    //     .route("/posts/{id}/vote", post(vote_post))
        
    //     .with_state(state)
    //     .layer(prom_layer)
    //     .layer(SetRequestIdLayer::x_request_id(make_req_id))
    //     .layer(PropagateRequestIdLayer::x_request_id())
    //     .layer(TraceLayer::new_for_http())
    //     .layer(middleware::from_fn(|req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| async move { next.run(req).await}))
    //     .layer(cors);

    // let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    // println!("Server running on http://0.0.0.0:3000");
    
    // axum::serve(listener, app).await?;
    // Ok(())
}

// Create COMMNET (atuh)
async fn create_comment(
    State(state): State<AppState>,
    AuthUser { user_id}: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(payload): Json<CreateCommentPayload>,
) -> std::result::Result<Json<CommentRow>, (StatusCode, Json<ApiError>)> {
    if let Err(e) = payload.validate() {
        return Err(http_err(StatusCode::BAD_REQUEST, &e.to_string()));
    }

    // ensure parent belongs to same post if provided 
    if let Some(pid) = payload.parent_id {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM comments WHERE id=$1 AND post_id=$2")
            .bind(pid).bind(post_id)
            .fetch_one(&state.db).await
            .map_err(|e| { error!(?e, "parent check failed"); http_err(StatusCode::INTERNAL_SERVER_ERROR, "internal error")})?;
        
        if exists == 0 { return Err(http_err(StatusCode::BAD_REQUEST, "parent comment not found in this post"));}
    }

    let id = Uuid::new_v4();
    let body = censor(&payload.body);

    let q = r#"
        INSERT INTO comments (id, post_id, user_id, parent_id, body)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, post_id, user_id, parent_id, body, created_at
    "#;

    let row = sqlx::query_as::<_, CommentRow>(q)
    .bind(id).bind(post_id).bind(user_id).bind(payload.parent_id).bind(body)
    .fetch_one(&state.db).await
    .map_err(|e| { error!(?e, "create_comment failed"); http_err(StatusCode::INTERNAL_SERVER_ERROR, "internal error")})?;

    Ok(Json(row))
}

// GET COMMENTS (nested tree)
async fn list_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> std::result::Result<Json<Vec<CommentNode>>, (StatusCode, Json<ApiError>)> {
    
    let rows: Vec<CommentRow> = sqlx::query_as::<_, CommentRow>(r#"
        SELECT id, post_id, user_id, parent_id, body, created_at
        FROM comments
        WHERE post_id = $1
        ORDER BY created_at ASC
    "#)
    .bind(post_id)
    .fetch_all(&state.db).await
    .map_err(|e| { error!(?e, "list_comments failed"); http_err(StatusCode::INTERNAL_SERVER_ERROR, "internal error")})?;

    // build tree in-memory
    let mut map: std::collections::HashMap<Uuid, CommentNode> = std::collections::HashMap::new();
    let mut roots: Vec<CommentNode> = vec![];

    for r in rows.into_iter() {
        map.insert(r.id, CommentNode { 
            id: r.id, 
            post_id: r.post_id, 
            user_id: r.user_id, 
            parent_id: r.parent_id, 
            body: r.body, 
            created_at: r.created_at, 
            children: vec![] 
        });
    }

    // second pass attach to parents
    let ids: Vec<Uuid> = map.keys().cloned().collect();
    for id in ids {
        if let Some(parent_id) = map[&id].parent_id {
            if let (Some(child), Some(parent)) = (map.remove(&id), map.get_mut(&parent_id)) {
                parent.children.push(child);
            }
        }
    }

    // remaining nodes in map are roots
    for (_, n) in map.into_iter() {
        if n.parent_id.is_none() { 
            roots.push(n);
        }
    }

    Ok(Json(roots))
}

async fn vote_post(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(payload): Json<VotePayload>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    if payload.value != 1 && payload.value != -1 {
        return Err(http_err(StatusCode::BAD_REQUEST, "value must be 1 or -1"));
    }

    // Upsert vote and recompute post score atomically.
    // score = sum of votes.value
    let q = r#"
        WITH upsert AS (
            INSERT INTO votes (user_id, post_id, value)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, post_id)
            DO UPDATE SET value = EXCLUDED.value
        )

        UPDATE posts p
        SET score = COALESCE(v.sum_value, 0)
        FROM (
            SELECT post_id, SUM(value)::int AS sum_value
            FROM votes
            WHERE post_id = $2
            GROUP BY post_id
        ) v
        WHERE p.id = v.post_id
        RETURNING p.id, p.score, p.created_at
    "#;

    #[derive(sqlx::FromRow)]
    struct ScoreRow {
        id: Uuid,
        score: i32,
        created_at: DateTime<Utc>,
    }

    let row = sqlx::query_as::<_, ScoreRow>(q)
    .bind(user_id)
    .bind(post_id)
    .bind(payload.value)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
            error!(error=?e, "vote query failed");
            http_err(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        })?
    .ok_or_else(|| http_err(StatusCode::NOT_FOUND, "post not found"))?;

    // Calculate "hot" score based on Reddit's algorithm
    // Simple version: score / (age_in_hours + 2)^1.8
    let age_hours = (Utc::now() - row.created_at).num_hours().max(0) as f64;
    let hot = row.score as f64 / (age_hours + 2.0).powf(1.8);

    Ok(Json(serde_json::json!({
        "post_id": row.id,
        "score": row.score,
        "hot": hot
    })))
}

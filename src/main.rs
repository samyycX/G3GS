use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

mod database;
mod id_encoder;
mod models;
mod queue;
mod redis_client;
mod service;

use crate::database::{create_pool, run_migrations};
use crate::redis_client::{create_redis_pool, RedisCache};
use crate::service::{create_shortlink, get_stats, redirect_to_url, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    // Database setup
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://g3gs:g3gs@localhost:5432/g3gs".to_string());
    
    let pool = create_pool(&database_url).await.unwrap();
    
    run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");

    // Redis setup
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let redis_pool = create_redis_pool(&redis_url).await?;
    let cache = RedisCache::new(redis_pool);

    // Queue setup
    let (access_sender, access_receiver) = queue::create_queue();
    
    // Start background task for processing access updates
    let pool_clone = Arc::new(pool.clone());
    tokio::spawn(async move {
        queue::process_access_updates(access_receiver, pool_clone).await;
    });

    // Domain configuration
    let domain = std::env::var("DOMAIN")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Ensure domain doesn't have trailing slash
    let domain = domain.trim_end_matches('/').to_string();

    // App state
    let state = AppState {
        pool: Arc::new(pool),
        cache: Arc::new(Mutex::new(cache)),
        access_sender,
        domain,
    };

    // CORS setup
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Static file service
    let serve_dir = ServeDir::new("frontend").not_found_service(
        ServeDir::new("frontend/_not-found").append_index_html_on_directories(true)
    );

    // Router setup
    let app = Router::new()
        .route("/api/shorten", post(create_shortlink))
        .route("/api/stats/{short_code}", get(get_stats))
        .route("/{short_code}", get(handle_shortcode_or_static))
        .fallback_service(serve_dir)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("G3GS server successfully started.");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_shortcode_or_static(
    State(state): State<AppState>,
    Path(short_code): Path<String>,
    headers: HeaderMap,
) -> Result<Redirect, StatusCode> {
    // 如果路径中包含点号，则认为是静态资源
    if short_code.contains('.') {
        return Ok(Redirect::permanent(format!("/public/{}", short_code).as_str()))
    }
    
    // 否则处理为短链接重定向
    redirect_to_url(State(state), Path(short_code), headers)
        .await
        .map_err(|(status, _)| status)
}

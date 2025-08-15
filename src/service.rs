use crate::id_encoder::{encode_id, decode_id};
use crate::models::{CreateShortlinkRequest, Shortlink, ShortlinkResponse};
use crate::queue::AccessUpdateSender;
use crate::redis_client::RedisCache;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
    Json,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
    pub cache: Arc<tokio::sync::Mutex<RedisCache>>,
    pub access_sender: AccessUpdateSender,
    pub domain: String,
}

pub async fn create_shortlink(
    State(state): State<AppState>,
    Json(payload): Json<CreateShortlinkRequest>,
) -> Result<Json<ShortlinkResponse>, (StatusCode, String)> {
    // Validate URL
    if !payload.url.starts_with("http://") && !payload.url.starts_with("https://") {
        return Err((StatusCode::BAD_REQUEST, "Invalid URL format".to_string()));
    }

    // Check if URL already exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM shortlinks WHERE original_url = $1"
    )
    .bind(&payload.url)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(id) = existing {
        let short_code = encode_id(id as u64);
        let short_url = format!("{}/{}", state.domain, short_code);
        
        let shortlink = sqlx::query_as::<_, Shortlink>(
            "SELECT * FROM shortlinks WHERE id = $1"
        )
        .bind(id)
        .fetch_one(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // if still valid
        let expires_at = shortlink.expires_at.unwrap();
        if expires_at.timestamp() == 0 || (Utc::now() - expires_at).to_std().is_err() {
            return Ok(Json(ShortlinkResponse {
                short_url,
                original_url: shortlink.original_url,
                created_at: shortlink.created_at,
                expires_at: shortlink.expires_at,
            }));
        };
    }

    
    let shortlink = sqlx::query_as::<_, Shortlink>(
        r#"
        INSERT INTO shortlinks (original_url, expires_at)
        VALUES ($1, $2)
        RETURNING id, original_url, created_at, expires_at, access_count
        "#
    )
    .bind(&payload.url)
    .bind(payload.expires_at)
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let short_code = encode_id(shortlink.id as u64);
    let short_url = format!("{}/{}", state.domain, short_code);

    // Cache the new shortlink
    let mut cache = state.cache.lock().await;
    let expires_in = payload.expires_at.map(|exp| {
        if exp.timestamp() == 0 {
            // 永不过期
            std::time::Duration::from_secs(365 * 24 * 60 * 60) // 1年作为Redis的最大过期时间
        } else {
            (exp - Utc::now()).to_std().unwrap_or_default()
        }
    });
    
    cache
        .set_url(&short_code.to_string(), &shortlink.original_url, expires_in)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(expires_at) = shortlink.expires_at {
        if expires_at.timestamp() != 0 {
            cache
                .set_expires_at(&short_code.to_string(), expires_at.timestamp())
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    Ok(Json(ShortlinkResponse {
        short_url,
        original_url: shortlink.original_url,
        created_at: shortlink.created_at,
        expires_at: shortlink.expires_at,
    }))
}

pub async fn redirect_to_url(
    State(state): State<AppState>,
    Path(short_code): Path<String>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, String)> {
    // Decode shortcode to id
    let id = decode_id(&short_code)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid short code".to_string()))?;

    // Check cache first
    let mut cache = state.cache.lock().await;
    
    // Check if expired
    if let Ok(Some(expires_at)) = cache.get_expires_at(&id.to_string()).await {
        if expires_at != 0 && expires_at < Utc::now().timestamp() {
            cache.delete_url(&id.to_string()).await.ok();
            return Err((StatusCode::NOT_FOUND, "Shortlink expired".to_string()));
        }
    }

    let url = match cache.get_url(&id.to_string()).await {
        Ok(Some(url)) => url,
        Ok(None) => {
            // Cache miss, fetch from database
            let shortlink = sqlx::query_as::<_, Shortlink>(
                "SELECT * FROM shortlinks WHERE id = $1"
            )
            .bind(id as i64)
            .fetch_optional(state.pool.as_ref())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or((StatusCode::NOT_FOUND, "Shortlink not found".to_string()))?;

            // Check expiration
            if let Some(expires_at) = shortlink.expires_at {
                if expires_at.timestamp() != 0 && expires_at < Utc::now() {
                    return Err((StatusCode::NOT_FOUND, "Shortlink expired".to_string()));
                }
                if expires_at.timestamp() != 0 {
                    cache
                        .set_expires_at(&id.to_string(), expires_at.timestamp())
                        .await
                        .ok();
                }
            }

            // Cache the URL
            let expires_in = shortlink.expires_at.map(|exp| {
                if exp.timestamp() == 0 {
                    // 永不过期
                    std::time::Duration::from_secs(365 * 24 * 60 * 60) // 1年作为Redis的最大过期时间
                } else {
                    (exp - Utc::now()).to_std().unwrap_or_default()
                }
            });
            cache
                .set_url(&id.to_string(), &shortlink.original_url, expires_in)
                .await
                .ok();

            shortlink.original_url
        }
        Err(_) => {
            // Redis error, fetch from database
            let shortlink = sqlx::query_as::<_, Shortlink>(
                "SELECT * FROM shortlinks WHERE id = $1"
            )
            .bind(id as i64)
            .fetch_optional(state.pool.as_ref())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or((StatusCode::NOT_FOUND, "Shortlink not found".to_string()))?;

            if let Some(expires_at) = shortlink.expires_at {
                if expires_at.timestamp() != 0 && expires_at < Utc::now() {
                    return Err((StatusCode::NOT_FOUND, "Shortlink expired".to_string()));
                }
            }

            shortlink.original_url
        }
    };

    // Send access update to queue
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let _ = state.access_sender.send(crate::queue::AccessUpdate {
        shortlink_id: id as i64,
        ip_address,
        user_agent,
    });

    Ok(Redirect::temporary(&url))
}

pub async fn get_stats(
    State(state): State<AppState>,
    Path(short_code): Path<String>,
) -> Result<Json<Shortlink>, (StatusCode, String)> {
    // Decode shortcode to id
    let id = decode_id(&short_code)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid short code".to_string()))?;

    let shortlink = sqlx::query_as::<_, Shortlink>(
        "SELECT * FROM shortlinks WHERE id = $1"
    )
    .bind(id as i64)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Shortlink not found".to_string()))?;

    Ok(Json(shortlink))
}
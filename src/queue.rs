use tokio::sync::mpsc;
use std::sync::Arc;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct AccessUpdate {
    pub shortlink_id: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub type AccessUpdateSender = mpsc::UnboundedSender<AccessUpdate>;
pub type AccessUpdateReceiver = mpsc::UnboundedReceiver<AccessUpdate>;

pub fn create_queue() -> (AccessUpdateSender, AccessUpdateReceiver) {
    mpsc::unbounded_channel()
}

pub async fn process_access_updates(
    mut receiver: AccessUpdateReceiver,
    pool: Arc<PgPool>,
) {
    while let Some(update) = receiver.recv().await {
        if let Err(e) = process_single_update(&update, &pool).await {
            tracing::error!("Failed to process access update: {}", e);
        }
    }
}

async fn process_single_update(
    update: &AccessUpdate,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    // Insert access log
    sqlx::query(
        r#"
        INSERT INTO access_logs (shortlink_id, ip_address, user_agent)
        VALUES ($1, $2::inet, $3)
        "#
    )
    .bind(update.shortlink_id)
    .bind(&update.ip_address)
    .bind(&update.user_agent)
    .execute(pool)
    .await?;

    // Update access count
    sqlx::query(
        r#"
        UPDATE shortlinks
        SET access_count = access_count + 1
        WHERE id = $1
        "#
    )
    .bind(update.shortlink_id)
    .execute(pool)
    .await?;

    Ok(())
}
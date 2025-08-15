use redis::{aio::ConnectionManager, Client};
use std::time::Duration;

pub type RedisPool = ConnectionManager;

pub async fn create_redis_pool(redis_url: &str) -> Result<RedisPool, redis::RedisError> {
    let client = Client::open(redis_url)?;
    ConnectionManager::new(client).await
}

pub struct RedisCache {
    pool: RedisPool,
}

impl RedisCache {
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    pub async fn get_url(&mut self, short_code: &str) -> Result<Option<String>, redis::RedisError> {
        let result: Option<String> = redis::cmd("GET")
            .arg(format!("short:{}", short_code))
            .query_async(&mut self.pool)
            .await?;
        Ok(result)
    }

    pub async fn set_url(
        &mut self,
        short_code: &str,
        url: &str,
        expires_in: Option<Duration>,
    ) -> Result<(), redis::RedisError> {
        let key = format!("short:{}", short_code);
        
        if let Some(duration) = expires_in {
            let _: () = redis::cmd("SETEX")
                .arg(key)
                .arg(duration.as_secs())
                .arg(url)
                .query_async(&mut self.pool)
                .await?;
        } else {
            let _: () = redis::cmd("SET")
                .arg(key)
                .arg(url)
                .query_async(&mut self.pool)
                .await?;
        }
        
        Ok(())
    }

    pub async fn get_expires_at(
        &mut self,
        short_code: &str,
    ) -> Result<Option<i64>, redis::RedisError> {
        let result: Option<i64> = redis::cmd("GET")
            .arg(format!("expires_at:{}", short_code))
            .query_async(&mut self.pool)
            .await?;
        Ok(result)
    }

    pub async fn set_expires_at(
        &mut self,
        short_code: &str,
        expires_at: i64,
    ) -> Result<(), redis::RedisError> {
        let key = format!("expires_at:{}", short_code);
        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(expires_at)
            .query_async(&mut self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_url(&mut self, short_code: &str) -> Result<(), redis::RedisError> {
        let _: () = redis::cmd("DEL")
            .arg(format!("short:{}", short_code))
            .arg(format!("expires_at:{}", short_code))
            .query_async(&mut self.pool)
            .await?;
        Ok(())
    }
}
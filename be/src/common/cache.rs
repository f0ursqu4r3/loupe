/// Redis-based caching layer
///
/// Provides distributed caching for:
/// - Dashboard metadata
/// - Query results (with TTL)
/// - Visualization configurations
/// - Datasource lists
/// - Organization membership

use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache manager with Redis backend
#[derive(Clone)]
pub struct CacheManager {
    client: ConnectionManager,
    enabled: bool,
    default_ttl: Duration,
}

impl CacheManager {
    /// Create a new cache manager
    ///
    /// # Environment variables
    /// - `REDIS_URL`: Redis connection string (default: "redis://localhost:6379")
    /// - `CACHE_ENABLED`: Enable/disable caching (default: "true")
    /// - `CACHE_DEFAULT_TTL`: Default TTL in seconds (default: "300" = 5 minutes)
    pub async fn new() -> Result<Self, RedisError> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let enabled = std::env::var("CACHE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let default_ttl_secs = std::env::var("CACHE_DEFAULT_TTL")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .unwrap_or(300);

        let client = redis::Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;

        tracing::info!(
            "Cache manager initialized (enabled: {}, default_ttl: {}s)",
            enabled,
            default_ttl_secs
        );

        Ok(Self {
            client: manager,
            enabled,
            default_ttl: Duration::from_secs(default_ttl_secs),
        })
    }

    /// Get a value from cache
    ///
    /// Returns `Ok(None)` if cache is disabled or key doesn't exist
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, RedisError>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.enabled {
            return Ok(None);
        }

        let mut conn = self.client.clone();
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(json_str) => {
                match serde_json::from_str(&json_str) {
                    Ok(data) => Ok(Some(data)),
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cached value for key '{}': {}", key, e);
                        // Delete corrupted cache entry
                        let _: () = conn.del(key).await?;
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with default TTL
    pub async fn set<T>(&self, key: &str, value: &T) -> Result<(), RedisError>
    where
        T: Serialize,
    {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value in cache with custom TTL
    pub async fn set_with_ttl<T>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), RedisError>
    where
        T: Serialize,
    {
        if !self.enabled {
            return Ok(());
        }

        let json_str = serde_json::to_string(value)
            .map_err(|e| RedisError::from((
                redis::ErrorKind::TypeError,
                "serialization error",
                e.to_string(),
            )))?;

        let mut conn = self.client.clone();
        let _: () = conn.set_ex(key, json_str, ttl.as_secs()).await?;

        Ok(())
    }

    /// Delete a key from cache
    pub async fn delete(&self, key: &str) -> Result<(), RedisError> {
        if !self.enabled {
            return Ok(());
        }

        let mut conn = self.client.clone();
        let _: () = conn.del(key).await?;

        Ok(())
    }

    /// Delete all keys matching a pattern
    ///
    /// Pattern examples:
    /// - `dashboards:*` - All dashboard cache entries
    /// - `dashboard:org:*` - All dashboards for any organization
    /// - `query:result:*` - All query result caches
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64, RedisError> {
        if !self.enabled {
            return Ok(0);
        }

        let mut conn = self.client.clone();

        // SCAN for keys matching pattern
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all matching keys
        let count = keys.len() as u64;
        let _: () = conn.del(&keys).await?;

        Ok(count)
    }

    /// Check if cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<CacheStats, RedisError> {
        let mut conn = self.client.clone();

        // Get Redis INFO stats
        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut conn)
            .await?;

        // Parse keyspace hits and misses
        let mut hits = 0u64;
        let mut misses = 0u64;

        for line in info.lines() {
            if line.starts_with("keyspace_hits:") {
                hits = line.split(':').nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("keyspace_misses:") {
                misses = line.split(':').nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
        }

        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(CacheStats {
            hits,
            misses,
            hit_rate,
            enabled: self.enabled,
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub enabled: bool,
}

/// Cache key builders for consistent naming
pub mod keys {
    use uuid::Uuid;

    /// Dashboard list for an organization
    pub fn dashboard_list(org_id: Uuid) -> String {
        format!("dashboards:org:{}", org_id)
    }

    /// Individual dashboard
    pub fn dashboard(id: Uuid) -> String {
        format!("dashboard:{}", id)
    }

    /// Query result
    pub fn query_result(run_id: Uuid) -> String {
        format!("query:result:{}", run_id)
    }

    /// Visualization list for a query
    pub fn visualization_list(query_id: Uuid) -> String {
        format!("visualizations:query:{}", query_id)
    }

    /// Individual visualization
    pub fn visualization(id: Uuid) -> String {
        format!("visualization:{}", id)
    }

    /// Datasource list for an organization
    pub fn datasource_list(org_id: Uuid) -> String {
        format!("datasources:org:{}", org_id)
    }

    /// User organization membership
    pub fn user_org(user_id: Uuid) -> String {
        format!("user:org:{}", user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_key_generation() {
        let org_id = Uuid::new_v4();
        let dashboard_id = Uuid::new_v4();

        assert_eq!(
            keys::dashboard_list(org_id),
            format!("dashboards:org:{}", org_id)
        );
        assert_eq!(
            keys::dashboard(dashboard_id),
            format!("dashboard:{}", dashboard_id)
        );
    }
}

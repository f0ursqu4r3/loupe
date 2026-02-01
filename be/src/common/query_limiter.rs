/// Query execution limiter to prevent resource exhaustion
///
/// Tracks concurrent query executions per organization and enforces limits.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Configuration for query execution limits
#[derive(Debug, Clone)]
pub struct QueryLimits {
    /// Maximum concurrent queries per organization
    pub max_concurrent_per_org: usize,
    /// Maximum concurrent queries globally
    pub max_concurrent_global: usize,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            max_concurrent_per_org: 10,
            max_concurrent_global: 100,
        }
    }
}

impl QueryLimits {
    /// Create limits from environment variables
    pub fn from_env() -> Self {
        let max_concurrent_per_org = std::env::var("MAX_CONCURRENT_QUERIES_PER_ORG")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let max_concurrent_global = std::env::var("MAX_CONCURRENT_QUERIES_GLOBAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        Self {
            max_concurrent_per_org,
            max_concurrent_global,
        }
    }
}

/// Tracks concurrent query executions
#[derive(Clone)]
pub struct QueryLimiter {
    limits: QueryLimits,
    state: Arc<Mutex<LimiterState>>,
}

struct LimiterState {
    /// Maps org_id -> count of running queries
    org_queries: HashMap<Uuid, usize>,
    /// Total running queries across all orgs
    total_queries: usize,
}

impl QueryLimiter {
    /// Create a new query limiter with default limits
    pub fn new(limits: QueryLimits) -> Self {
        Self {
            limits,
            state: Arc::new(Mutex::new(LimiterState {
                org_queries: HashMap::new(),
                total_queries: 0,
            })),
        }
    }

    /// Attempt to acquire a slot for query execution
    ///
    /// Returns Ok(QueryGuard) if the query can proceed, or Err if limit is reached.
    pub fn try_acquire(&self, org_id: Uuid) -> Result<QueryGuard, LimitError> {
        let mut state = self.state.lock().unwrap();

        // Check global limit
        if state.total_queries >= self.limits.max_concurrent_global {
            return Err(LimitError::GlobalLimitReached {
                current: state.total_queries,
                max: self.limits.max_concurrent_global,
            });
        }

        // Check per-org limit
        let org_count = state.org_queries.get(&org_id).copied().unwrap_or(0);
        if org_count >= self.limits.max_concurrent_per_org {
            return Err(LimitError::OrgLimitReached {
                org_id,
                current: org_count,
                max: self.limits.max_concurrent_per_org,
            });
        }

        // Increment counters
        state.total_queries += 1;
        *state.org_queries.entry(org_id).or_insert(0) += 1;

        Ok(QueryGuard {
            org_id,
            limiter: self.clone(),
        })
    }

    /// Release a query slot (called automatically when QueryGuard is dropped)
    fn release(&self, org_id: Uuid) {
        let mut state = self.state.lock().unwrap();

        if let Some(count) = state.org_queries.get_mut(&org_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.org_queries.remove(&org_id);
            }
        }

        state.total_queries = state.total_queries.saturating_sub(1);
    }

    /// Get current statistics
    pub fn stats(&self) -> LimiterStats {
        let state = self.state.lock().unwrap();
        LimiterStats {
            total_queries: state.total_queries,
            org_count: state.org_queries.len(),
            max_concurrent_global: self.limits.max_concurrent_global,
            max_concurrent_per_org: self.limits.max_concurrent_per_org,
        }
    }
}

/// Guard that automatically releases a query slot when dropped
pub struct QueryGuard {
    org_id: Uuid,
    limiter: QueryLimiter,
}

impl Drop for QueryGuard {
    fn drop(&mut self) {
        self.limiter.release(self.org_id);
    }
}

/// Statistics about current query execution
#[derive(Debug, Clone)]
pub struct LimiterStats {
    pub total_queries: usize,
    pub org_count: usize,
    pub max_concurrent_global: usize,
    pub max_concurrent_per_org: usize,
}

/// Errors that can occur when acquiring a query slot
#[derive(Debug, thiserror::Error)]
pub enum LimitError {
    #[error("Global query limit reached: {current}/{max} queries running")]
    GlobalLimitReached { current: usize, max: usize },

    #[error("Organization query limit reached for {org_id}: {current}/{max} queries running")]
    OrgLimitReached {
        org_id: Uuid,
        current: usize,
        max: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_basic() {
        let limits = QueryLimits {
            max_concurrent_per_org: 2,
            max_concurrent_global: 5,
        };
        let limiter = QueryLimiter::new(limits);
        let org_id = Uuid::new_v4();

        // Should be able to acquire up to the limit
        let _guard1 = limiter.try_acquire(org_id).unwrap();
        let _guard2 = limiter.try_acquire(org_id).unwrap();

        // Third should fail
        assert!(matches!(
            limiter.try_acquire(org_id),
            Err(LimitError::OrgLimitReached { .. })
        ));

        // Stats should reflect current state
        let stats = limiter.stats();
        assert_eq!(stats.total_queries, 2);
    }

    #[test]
    fn test_limiter_release() {
        let limits = QueryLimits {
            max_concurrent_per_org: 2,
            max_concurrent_global: 5,
        };
        let limiter = QueryLimiter::new(limits);
        let org_id = Uuid::new_v4();

        {
            let _guard = limiter.try_acquire(org_id).unwrap();
            assert_eq!(limiter.stats().total_queries, 1);
        } // Guard dropped here

        // Should be released
        assert_eq!(limiter.stats().total_queries, 0);
    }

    #[test]
    fn test_global_limit() {
        let limits = QueryLimits {
            max_concurrent_per_org: 10,
            max_concurrent_global: 2,
        };
        let limiter = QueryLimiter::new(limits);

        let org1 = Uuid::new_v4();
        let org2 = Uuid::new_v4();

        let _guard1 = limiter.try_acquire(org1).unwrap();
        let _guard2 = limiter.try_acquire(org2).unwrap();

        // Third should fail due to global limit
        assert!(matches!(
            limiter.try_acquire(org1),
            Err(LimitError::GlobalLimitReached { .. })
        ));
    }
}

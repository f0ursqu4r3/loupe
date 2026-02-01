use serde::{Deserialize, Serialize};

/// Default page size for paginated responses
pub const DEFAULT_PAGE_SIZE: i64 = 20;

/// Maximum page size to prevent excessive data transfer
pub const MAX_PAGE_SIZE: i64 = 100;

/// Pagination parameters from query string
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PaginationParams {
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_limit")]
    pub limit: i64,

    /// Number of items to skip (default: 0)
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    DEFAULT_PAGE_SIZE
}

impl PaginationParams {
    /// Create new pagination params with validation
    pub fn new(limit: Option<i64>, offset: Option<i64>) -> Self {
        let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE).max(1);
        let offset = offset.unwrap_or(0).max(0);

        Self { limit, offset }
    }

    /// Validate and normalize the pagination parameters
    pub fn validate(&mut self) {
        // Ensure limit is within bounds
        if self.limit < 1 {
            self.limit = DEFAULT_PAGE_SIZE;
        }
        if self.limit > MAX_PAGE_SIZE {
            self.limit = MAX_PAGE_SIZE;
        }

        // Ensure offset is non-negative
        if self.offset < 0 {
            self.offset = 0;
        }
    }

    /// Get the current page number (1-indexed)
    pub fn page(&self) -> i64 {
        (self.offset / self.limit) + 1
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: DEFAULT_PAGE_SIZE,
            offset: 0,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// The data items for the current page
    pub data: Vec<T>,

    /// Total number of items across all pages
    pub total: i64,

    /// Current page number (1-indexed)
    pub page: i64,

    /// Number of items per page
    pub per_page: i64,

    /// Total number of pages
    pub total_pages: i64,

    /// Whether there is a next page
    pub has_next: bool,

    /// Whether there is a previous page
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, total: i64, params: &PaginationParams) -> Self {
        let total_pages = if params.limit > 0 {
            (total + params.limit - 1) / params.limit // Ceiling division
        } else {
            0
        };

        let page = params.page();
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            data,
            total,
            page,
            per_page: params.limit,
            total_pages,
            has_next,
            has_prev,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::default();
        assert_eq!(params.limit, DEFAULT_PAGE_SIZE);
        assert_eq!(params.offset, 0);
        assert_eq!(params.page(), 1);
    }

    #[test]
    fn test_pagination_params_validate() {
        let mut params = PaginationParams {
            limit: 200, // Exceeds max
            offset: -10, // Negative
        };
        params.validate();

        assert_eq!(params.limit, MAX_PAGE_SIZE);
        assert_eq!(params.offset, 0);
    }

    #[test]
    fn test_pagination_params_new() {
        let params = PaginationParams::new(Some(50), Some(100));
        assert_eq!(params.limit, 50);
        assert_eq!(params.offset, 100);
        assert_eq!(params.page(), 3); // offset 100 / limit 50 = page 3
    }

    #[test]
    fn test_pagination_params_page_calculation() {
        let params = PaginationParams {
            limit: 20,
            offset: 0,
        };
        assert_eq!(params.page(), 1);

        let params = PaginationParams {
            limit: 20,
            offset: 20,
        };
        assert_eq!(params.page(), 2);

        let params = PaginationParams {
            limit: 20,
            offset: 40,
        };
        assert_eq!(params.page(), 3);
    }

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3, 4, 5];
        let params = PaginationParams {
            limit: 5,
            offset: 0,
        };

        let response = PaginatedResponse::new(items, 23, &params);

        assert_eq!(response.total, 23);
        assert_eq!(response.page, 1);
        assert_eq!(response.per_page, 5);
        assert_eq!(response.total_pages, 5);  // 23 / 5 = 4.6 -> 5 pages
        assert!(response.has_next);
        assert!(!response.has_prev);
    }

    #[test]
    fn test_paginated_response_last_page() {
        let items = vec![1, 2, 3];
        let params = PaginationParams {
            limit: 5,
            offset: 20, // Last page
        };

        let response = PaginatedResponse::new(items, 23, &params);

        assert_eq!(response.page, 5);
        assert!(!response.has_next);
        assert!(response.has_prev);
    }
}

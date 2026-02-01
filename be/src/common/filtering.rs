use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Sort parameters - reusable across all endpoints
#[derive(Debug, Clone, Deserialize)]
pub struct SortParams {
    /// Column to sort by (validated against whitelist)
    pub sort_by: Option<String>,

    /// Sort direction: "asc" or "desc" (default: "desc")
    pub sort_direction: Option<String>,
}

impl SortParams {
    /// Validate and get SQL ORDER BY components
    /// Returns (column, direction) tuple with defaults if invalid
    pub fn validate_and_build(
        &self,
        allowed_columns: &[&str],
        default_column: &str,
    ) -> (String, String) {
        // Validate column against whitelist
        let column = self
            .sort_by
            .as_ref()
            .filter(|col| allowed_columns.contains(&col.as_str()))
            .map(|s| s.to_string())
            .unwrap_or_else(|| default_column.to_string());

        // Validate direction
        let direction = self
            .sort_direction
            .as_ref()
            .and_then(|d| {
                let lower = d.to_lowercase();
                if lower == "asc" || lower == "desc" {
                    Some(lower.to_uppercase())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "DESC".to_string());

        (column, direction)
    }
}

/// Text search parameter (ILIKE for PostgreSQL)
#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    /// Search term for text fields
    pub search: Option<String>,
}

impl SearchParams {
    /// Get sanitized search pattern for ILIKE query
    /// Returns None if empty/whitespace-only or too long
    pub fn get_pattern(&self) -> Option<String> {
        self.search
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter(|s| s.len() <= 200) // Max length for search term
            .map(|s| format!("%{}%", s))
    }
}

/// Date range filter
#[derive(Debug, Clone, Deserialize)]
pub struct DateRangeParams {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Parse comma-separated tags into a Vec
/// Filters out empty tags and enforces limits
pub fn parse_tags(tags_csv: &str) -> Vec<String> {
    tags_csv
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() <= 50) // Max tag length
        .take(10) // Max 10 tags per filter
        .collect()
}

/// Sortable column whitelists for each endpoint
pub struct SortableColumns;

impl SortableColumns {
    pub const DASHBOARDS: &'static [&'static str] =
        &["name", "created_at", "updated_at"];

    pub const QUERIES: &'static [&'static str] =
        &["name", "created_at", "updated_at"];

    pub const RUNS: &'static [&'static str] =
        &["created_at", "started_at", "completed_at"];

    pub const VISUALIZATIONS: &'static [&'static str] =
        &["name", "created_at", "updated_at"];

    pub const SCHEDULES: &'static [&'static str] =
        &["name", "next_run_at", "created_at", "updated_at"];

    pub const DATASOURCES: &'static [&'static str] =
        &["name", "created_at", "updated_at"];

    pub const CANVASES: &'static [&'static str] =
        &["name", "created_at", "updated_at"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_params_valid() {
        let params = SortParams {
            sort_by: Some("name".to_string()),
            sort_direction: Some("asc".to_string()),
        };

        let allowed = &["name", "created_at"];
        let (col, dir) = params.validate_and_build(allowed, "created_at");

        assert_eq!(col, "name");
        assert_eq!(dir, "ASC");
    }

    #[test]
    fn test_sort_params_sql_injection_prevention() {
        let params = SortParams {
            sort_by: Some("name; DROP TABLE users;".to_string()),
            sort_direction: Some("asc".to_string()),
        };

        let allowed = &["name", "created_at"];
        let (col, dir) = params.validate_and_build(allowed, "created_at");

        // Should fall back to default when injection attempted
        assert_eq!(col, "created_at");
        assert_eq!(dir, "ASC");
    }

    #[test]
    fn test_sort_params_case_insensitive_direction() {
        let params = SortParams {
            sort_by: Some("name".to_string()),
            sort_direction: Some("AsC".to_string()),
        };

        let allowed = &["name"];
        let (col, dir) = params.validate_and_build(allowed, "created_at");

        assert_eq!(col, "name");
        assert_eq!(dir, "ASC");
    }

    #[test]
    fn test_sort_params_invalid_direction() {
        let params = SortParams {
            sort_by: Some("name".to_string()),
            sort_direction: Some("invalid".to_string()),
        };

        let allowed = &["name"];
        let (col, dir) = params.validate_and_build(allowed, "created_at");

        assert_eq!(col, "name");
        assert_eq!(dir, "DESC"); // Falls back to default
    }

    #[test]
    fn test_sort_params_defaults() {
        let params = SortParams {
            sort_by: None,
            sort_direction: None,
        };

        let allowed = &["name", "created_at"];
        let (col, dir) = params.validate_and_build(allowed, "created_at");

        assert_eq!(col, "created_at");
        assert_eq!(dir, "DESC");
    }

    #[test]
    fn test_search_params_valid() {
        let params = SearchParams {
            search: Some("analytics".to_string()),
        };

        let pattern = params.get_pattern();
        assert_eq!(pattern, Some("%analytics%".to_string()));
    }

    #[test]
    fn test_search_params_empty() {
        let params = SearchParams {
            search: Some("   ".to_string()),
        };

        let pattern = params.get_pattern();
        assert_eq!(pattern, None);
    }

    #[test]
    fn test_search_params_too_long() {
        let params = SearchParams {
            search: Some("a".repeat(201)),
        };

        let pattern = params.get_pattern();
        assert_eq!(pattern, None);
    }

    #[test]
    fn test_search_params_trimmed() {
        let params = SearchParams {
            search: Some("  analytics  ".to_string()),
        };

        let pattern = params.get_pattern();
        assert_eq!(pattern, Some("%analytics%".to_string()));
    }

    #[test]
    fn test_parse_tags_valid() {
        let tags = parse_tags("prod,analytics,dashboard");
        assert_eq!(tags, vec!["prod", "analytics", "dashboard"]);
    }

    #[test]
    fn test_parse_tags_with_whitespace() {
        let tags = parse_tags(" prod , analytics , dashboard ");
        assert_eq!(tags, vec!["prod", "analytics", "dashboard"]);
    }

    #[test]
    fn test_parse_tags_empty() {
        let tags = parse_tags("");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_parse_tags_filters_empty() {
        let tags = parse_tags("prod,,analytics,,");
        assert_eq!(tags, vec!["prod", "analytics"]);
    }

    #[test]
    fn test_parse_tags_max_limit() {
        let input = (0..15).map(|i| format!("tag{}", i)).collect::<Vec<_>>().join(",");
        let tags = parse_tags(&input);
        assert_eq!(tags.len(), 10); // Max 10 tags
    }

    #[test]
    fn test_parse_tags_length_limit() {
        let long_tag = "a".repeat(51);
        let tags = parse_tags(&format!("valid,{}", long_tag));
        assert_eq!(tags, vec!["valid"]); // Long tag filtered out
    }
}

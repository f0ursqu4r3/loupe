//! Property-based tests for security-critical paths
//!
//! These tests fuzz critical security components to find edge cases and vulnerabilities

use proptest::prelude::*;
use loupe::validation::*;

// ============================================================================
// SQL Validation Fuzzing
// ============================================================================

proptest! {
    #[test]
    fn test_sql_validation_accepts_valid(
        sql in "[A-Z ]{10,1000}"
    ) {
        assert!(validate_sql_length(&sql).is_ok());
    }

    #[test]
    fn test_sql_validation_rejects_too_long(
        prefix in "[A-Z ]{10,100}"
    ) {
        let too_long = prefix + &"A".repeat(100_001);
        assert!(validate_sql_length(&too_long).is_err());
    }

    #[test]
    fn test_sql_validation_handles_unicode(
        sql in "SELECT [😀-😅]{1,10} FROM users"
    ) {
        // Should accept or reject gracefully, not panic
        let _ = validate_sql_length(&sql);
    }
}

// ============================================================================
// Connection String Validation Fuzzing
// ============================================================================

proptest! {
    #[test]
    fn test_connection_string_requires_scheme(
        host in "[a-z]{3,20}",
        port in 1000u16..65535u16,
    ) {
        let no_scheme = format!("{}:{}", host, port);
        assert!(validate_connection_string(&no_scheme).is_err());

        let with_scheme = format!("postgresql://{}:{}/db", host, port);
        assert!(validate_connection_string(&with_scheme).is_ok());
    }

    #[test]
    fn test_connection_string_rejects_sql_injection(
        prefix in "postgresql://[a-z]{3,10}",
        injection in prop_oneof![
            Just("';DROP TABLE users--"),
            Just("/**/;exec('"),
            Just("xp_cmdshell"),
            Just("sp_executesql"),
        ],
    ) {
        let malicious = format!("{}{}", prefix, injection);
        assert!(validate_connection_string(&malicious).is_err());
    }

    #[test]
    fn test_connection_string_valid_formats(
        host in "[a-z0-9.-]{5,50}",
        port in 1000u16..65535u16,
        db in "[a-z0-9_]{3,20}",
    ) {
        let conn_str = format!("postgresql://{}:{}/{}", host, port, db);
        if conn_str.len() <= 2048 && conn_str.len() >= 10 {
            assert!(validate_connection_string(&conn_str).is_ok());
        }
    }
}

// ============================================================================
// Name Validation Fuzzing
// ============================================================================

proptest! {
    #[test]
    fn test_name_validation_allowed_chars(
        name in "[a-zA-Z0-9 _.-]{1,255}"
    ) {
        assert!(validate_name(&name).is_ok());
    }

    #[test]
    fn test_name_validation_rejects_special_chars(
        prefix in "[a-zA-Z]{3,10}",
        special in prop_oneof![
            Just("$"), Just("@"), Just("#"), Just("%"),
            Just("^"), Just("&"), Just("*"), Just("!"),
        ],
    ) {
        let invalid_name = format!("{}{}", prefix, special);
        assert!(validate_name(&invalid_name).is_err());
    }
}

// ============================================================================
// Pagination Validation Fuzzing
// ============================================================================

proptest! {
    #[test]
    fn test_pagination_valid_limits(
        limit in 1i64..=100i64,
        offset in 0i64..10000i64,
    ) {
        assert!(validate_pagination(limit, offset).is_ok());
    }

    #[test]
    fn test_pagination_invalid_limits(
        limit in prop_oneof![
            -1000i64..0i64,
            101i64..1000i64,
        ],
    ) {
        assert!(validate_pagination(limit, 0).is_err());
    }

    #[test]
    fn test_pagination_negative_offset(
        offset in -1000i64..-1i64,
    ) {
        assert!(validate_pagination(20, offset).is_err());
    }
}

// ============================================================================
// Date Range Validation Fuzzing
// ============================================================================

proptest! {
    #[test]
    fn test_date_range_validation_valid(
        days_offset in 1i64..3650i64,
    ) {
        use chrono::Utc;
        let start = Utc::now();
        let end = start + chrono::Duration::days(days_offset);
        assert!(validate_date_range(Some(start), Some(end)).is_ok());
    }

    #[test]
    fn test_date_range_validation_invalid(
        days_back in 1i64..365i64,
    ) {
        use chrono::Utc;
        let end = Utc::now();
        let start = end + chrono::Duration::days(days_back);
        assert!(validate_date_range(Some(start), Some(end)).is_err());
    }

    #[test]
    fn test_date_range_validation_too_large(
        days_offset in 3651i64..10000i64,
    ) {
        use chrono::Utc;
        let start = Utc::now();
        let end = start + chrono::Duration::days(days_offset);
        assert!(validate_date_range(Some(start), Some(end)).is_err());
    }
}

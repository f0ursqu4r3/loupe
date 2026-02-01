//! Property-based tests for models
//!
//! These tests use proptest to automatically generate test cases and verify invariants

use proptest::prelude::*;
use loupe::models::*;
use uuid::Uuid;
use chrono::Utc;
use serde_json;

// ============================================================================
// Strategy Generators - Define how to generate random test data
// ============================================================================

/// Generate arbitrary OrgRole values
fn arb_org_role() -> impl Strategy<Value = OrgRole> {
    prop_oneof![
        Just(OrgRole::Admin),
        Just(OrgRole::Editor),
        Just(OrgRole::Viewer),
    ]
}

/// Generate arbitrary DatasourceType values
fn arb_datasource_type() -> impl Strategy<Value = DatasourceType> {
    Just(DatasourceType::Postgres)
}

/// Generate arbitrary ParamType values
fn arb_param_type() -> impl Strategy<Value = ParamType> {
    prop_oneof![
        Just(ParamType::String),
        Just(ParamType::Number),
        Just(ParamType::Boolean),
        Just(ParamType::Date),
        Just(ParamType::DateTime),
    ]
}

/// Generate arbitrary RunStatus values
fn arb_run_status() -> impl Strategy<Value = RunStatus> {
    prop_oneof![
        Just(RunStatus::Queued),
        Just(RunStatus::Running),
        Just(RunStatus::Completed),
        Just(RunStatus::Failed),
        Just(RunStatus::Cancelled),
        Just(RunStatus::Timeout),
    ]
}

/// Generate arbitrary ChartType values
fn arb_chart_type() -> impl Strategy<Value = ChartType> {
    prop_oneof![
        Just(ChartType::Table),
        Just(ChartType::Line),
        Just(ChartType::Bar),
        Just(ChartType::Pie),
        Just(ChartType::SingleStat),
    ]
}

/// Generate valid email addresses
fn arb_email() -> impl Strategy<Value = String> {
    "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)".prop_map(|s| s.to_lowercase())
}

/// Generate valid SQL queries
fn arb_sql() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("SELECT 1".to_string()),
        Just("SELECT * FROM users".to_string()),
        Just("SELECT COUNT(*) FROM events WHERE date > $1".to_string()),
        Just("INSERT INTO logs (message) VALUES ($1)".to_string()),
        Just("UPDATE settings SET value = $1 WHERE key = $2".to_string()),
    ]
}

/// Generate reasonable timeout values (1-300 seconds)
fn arb_timeout() -> impl Strategy<Value = i32> {
    1..=300i32
}

/// Generate reasonable max_rows values (1-100000)
fn arb_max_rows() -> impl Strategy<Value = i32> {
    1..=100000i32
}

/// Generate arbitrary ParamDef
fn arb_param_def() -> impl Strategy<Value = ParamDef> {
    ("[a-z]{3,20}", arb_param_type(), proptest::bool::ANY).prop_map(|(name, param_type, required)| {
        ParamDef {
            name,
            param_type,
            default: if required { None } else { Some(serde_json::json!(null)) },
            required,
        }
    })
}

// ============================================================================
// Enum Serialization/Deserialization Invariants
// ============================================================================

proptest! {
    /// OrgRole should roundtrip through JSON serialization
    #[test]
    fn test_org_role_roundtrip(role in arb_org_role()) {
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: OrgRole = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(role, deserialized);
    }

    /// DatasourceType should roundtrip through JSON serialization
    #[test]
    fn test_datasource_type_roundtrip(ds_type in arb_datasource_type()) {
        let json = serde_json::to_string(&ds_type).unwrap();
        let deserialized: DatasourceType = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(ds_type, deserialized);
    }

    /// ParamType should serialize/deserialize through JSON
    #[test]
    fn test_param_type_serialization(param_type in arb_param_type()) {
        let json = serde_json::to_string(&param_type).unwrap();
        // Should deserialize without error
        let _deserialized: ParamType = serde_json::from_str(&json).unwrap();
        // Verify it produces valid JSON
        prop_assert!(!json.is_empty());
        prop_assert!(json.starts_with('"'));
        prop_assert!(json.ends_with('"'));
    }

    /// RunStatus should roundtrip through JSON serialization
    #[test]
    fn test_run_status_roundtrip(status in arb_run_status()) {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: RunStatus = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(status, deserialized);
    }

    /// ChartType should roundtrip through JSON serialization
    #[test]
    fn test_chart_type_roundtrip(chart_type in arb_chart_type()) {
        let json = serde_json::to_string(&chart_type).unwrap();
        let deserialized: ChartType = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(chart_type, deserialized);
    }
}

// ============================================================================
// Request Model Validation Invariants
// ============================================================================

proptest! {
    /// CreateQueryRequest should have valid timeout and max_rows
    #[test]
    fn test_create_query_request_invariants(
        name in "[a-zA-Z0-9 ]{3,100}",
        sql in arb_sql(),
        timeout in arb_timeout(),
        max_rows in arb_max_rows(),
    ) {
        let name_len = name.len();
        let req = CreateQueryRequest {
            datasource_id: Uuid::new_v4(),
            name,
            description: Some("Test query".to_string()),
            sql,
            parameters: vec![],
            tags: vec![],
            timeout_seconds: timeout,
            max_rows,
        };

        // Invariants
        prop_assert!(req.timeout_seconds > 0);
        prop_assert!(req.timeout_seconds <= 300);
        prop_assert!(req.max_rows > 0);
        prop_assert!(req.max_rows <= 100000);
        prop_assert!(!req.name.is_empty());
        prop_assert!(!req.sql.is_empty());

        // Verify fields are set correctly
        prop_assert_eq!(req.name.len(), name_len);
        prop_assert_eq!(req.timeout_seconds, timeout);
        prop_assert_eq!(req.max_rows, max_rows);
    }

    /// ExecuteAdHocRequest should have valid timeout and max_rows
    #[test]
    fn test_execute_adhoc_request_invariants(
        sql in arb_sql(),
        timeout in arb_timeout(),
        max_rows in arb_max_rows(),
    ) {
        let req = ExecuteAdHocRequest {
            datasource_id: Uuid::new_v4(),
            sql,
            parameters: serde_json::json!({}),
            timeout_seconds: timeout,
            max_rows,
        };

        prop_assert!(req.timeout_seconds > 0);
        prop_assert!(req.timeout_seconds <= 300);
        prop_assert!(req.max_rows > 0);
        prop_assert!(req.max_rows <= 100000);
    }

    /// CreateDatasourceRequest should have non-empty name and connection_string
    #[test]
    fn test_create_datasource_request_invariants(
        name in "[a-zA-Z0-9 ]{3,100}",
        ds_type in arb_datasource_type(),
    ) {
        let req = CreateDatasourceRequest {
            name: name.clone(),
            ds_type,
            connection_string: "postgres://localhost:5432/test".to_string(),
        };

        prop_assert!(!req.name.is_empty());
        prop_assert!(!req.connection_string.is_empty());
        prop_assert!(req.name.len() <= 255);

        // Verify fields are set correctly
        prop_assert_eq!(req.name, name);
    }

    /// ParamDef should maintain invariants
    #[test]
    fn test_param_def_invariants(param_def in arb_param_def()) {
        // Name should not be empty
        prop_assert!(!param_def.name.is_empty());

        // If required, default should be None
        if param_def.required {
            prop_assert!(param_def.default.is_none() || param_def.default == Some(serde_json::json!(null)));
        }

        // Roundtrip
        let json = serde_json::to_string(&param_def).unwrap();
        let deserialized: ParamDef = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(param_def.name, deserialized.name);
        // Note: ParamType doesn't implement PartialEq, so we just verify deserialization succeeds
        prop_assert_eq!(param_def.required, deserialized.required);
    }
}

// ============================================================================
// Response Model Security Invariants
// ============================================================================

proptest! {
    /// UserResponse should NEVER contain password_hash
    #[test]
    fn test_user_response_excludes_password(
        email in arb_email(),
        name in "[a-zA-Z ]{3,50}",
        role in arb_org_role(),
    ) {
        let user = User {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            email: email.clone(),
            password_hash: "super_secret_hash_12345".to_string(),
            name: name.clone(),
            role,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = UserResponse::from(user);
        let json = serde_json::to_string(&response).unwrap();

        // CRITICAL: password_hash must NEVER appear in serialized response
        prop_assert!(!json.contains("super_secret_hash"));
        prop_assert!(!json.contains("password_hash"));
        prop_assert!(!json.contains("password"));

        // Response should contain expected fields
        prop_assert!(json.contains(&email));
        prop_assert!(json.contains(&name));
    }

    /// DatasourceResponse should NEVER contain connection_string
    #[test]
    fn test_datasource_response_excludes_connection_string(
        name in "[a-zA-Z0-9 ]{3,100}",
        ds_type in arb_datasource_type(),
    ) {
        let ds = Datasource {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            name: name.clone(),
            ds_type,
            connection_string_encrypted: "ENCRYPTED_SECRET_CONNECTION_STRING".to_string(),
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = DatasourceResponse::from(ds);
        let json = serde_json::to_string(&response).unwrap();

        // CRITICAL: connection_string must NEVER appear in response
        prop_assert!(!json.contains("ENCRYPTED_SECRET_CONNECTION_STRING"));
        prop_assert!(!json.contains("connection_string"));

        // Response should contain expected fields
        prop_assert!(json.contains(&name));
    }
}

// ============================================================================
// Data Integrity Invariants
// ============================================================================

proptest! {
    /// RunResult should have consistent row_count with actual rows
    #[test]
    fn test_run_result_row_count_consistency(
        row_count in 0..100usize,
    ) {
        let rows: Vec<Vec<serde_json::Value>> = (0..row_count)
            .map(|i| vec![serde_json::json!(i)])
            .collect();

        let result = RunResult {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            columns: serde_json::json!([{"name": "id", "data_type": "INT8"}]),
            rows: serde_json::to_value(&rows).unwrap(),
            row_count: row_count as i64,
            byte_count: 100,
            execution_time_ms: 50,
            created_at: Utc::now(),
            expires_at: None,
        };

        let response = RunResultResponse::from(result);

        // Row count should match actual rows
        prop_assert_eq!(response.row_count as usize, row_count);
        prop_assert_eq!(response.rows.len(), row_count);
    }

    /// CreateTileRequest should have valid dimensions
    #[test]
    fn test_create_tile_request_dimensions(
        pos_x in 0..24i32,
        pos_y in 0..100i32,
        width in 1..12i32,
        height in 1..20i32,
    ) {
        let req = CreateTileRequest {
            visualization_id: Uuid::new_v4(),
            title: Some("Test Tile".to_string()),
            pos_x,
            pos_y,
            width,
            height,
            parameter_bindings: serde_json::json!({}),
        };

        // Dimensions should be positive
        prop_assert!(req.width > 0);
        prop_assert!(req.height > 0);

        // Position should be non-negative
        prop_assert!(req.pos_x >= 0);
        prop_assert!(req.pos_y >= 0);

        // Reasonable bounds for grid layout (12-column grid)
        prop_assert!(req.width <= 12);
    }

    /// CreateScheduleRequest should have valid cron expression format
    #[test]
    fn test_create_schedule_request_cron_format(
        name in "[a-zA-Z0-9 ]{3,100}",
    ) {
        // Common cron patterns
        let cron_expressions = vec![
            "0 * * * *",      // Every hour
            "0 0 * * *",      // Daily at midnight
            "0 0 * * 0",      // Weekly on Sunday
            "0 0 1 * *",      // Monthly on 1st
            "*/5 * * * *",    // Every 5 minutes
        ];

        for cron_expr in cron_expressions {
            let req = CreateScheduleRequest {
                query_id: Uuid::new_v4(),
                name: name.clone(),
                cron_expression: cron_expr.to_string(),
                parameters: serde_json::json!({}),
                tags: vec![],
                enabled: true,
            };

            // Cron expression should not be empty
            prop_assert!(!req.cron_expression.is_empty());

            // Should have 5 fields (minute hour day month weekday)
            let parts: Vec<&str> = req.cron_expression.split_whitespace().collect();
            prop_assert_eq!(parts.len(), 5);
        }
    }
}

// ============================================================================
// Edge Cases and Boundary Conditions
// ============================================================================

proptest! {
    /// Test empty and minimal values for query names
    #[test]
    fn test_query_name_edge_cases(
        name in ".|[a-zA-Z0-9 ]{1,255}",
    ) {
        if !name.is_empty() && name.len() <= 255 {
            let req = CreateQueryRequest {
                datasource_id: Uuid::new_v4(),
                name,
                description: None,
                sql: "SELECT 1".to_string(),
                parameters: vec![],
                tags: vec![],
                timeout_seconds: 30,
                max_rows: 1000,
            };

            // Verify the request was created with the correct name
            prop_assert!(!req.name.is_empty());
            prop_assert!(req.name.len() <= 255);
        }
    }

    /// Test boundary values for timeout_seconds
    #[test]
    fn test_timeout_boundaries(timeout in 1..=300i32) {
        let req = CreateQueryRequest {
            datasource_id: Uuid::new_v4(),
            name: "Test".to_string(),
            description: None,
            sql: "SELECT 1".to_string(),
            parameters: vec![],
            tags: vec![],
            timeout_seconds: timeout,
            max_rows: 1000,
        };

        // Should accept any timeout in valid range
        prop_assert!(req.timeout_seconds >= 1);
        prop_assert!(req.timeout_seconds <= 300);
    }

    /// Test boundary values for max_rows
    #[test]
    fn test_max_rows_boundaries(max_rows in 1..=100000i32) {
        let req = CreateQueryRequest {
            datasource_id: Uuid::new_v4(),
            name: "Test".to_string(),
            description: None,
            sql: "SELECT 1".to_string(),
            parameters: vec![],
            tags: vec![],
            timeout_seconds: 30,
            max_rows,
        };

        // Should accept any max_rows in valid range
        prop_assert!(req.max_rows >= 1);
        prop_assert!(req.max_rows <= 100000);
    }
}

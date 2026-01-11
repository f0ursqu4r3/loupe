//! Unit tests for models

mod user_tests {
    use crate::models::{OrgRole, UserResponse, User};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_org_role_default() {
        let role: OrgRole = Default::default();
        assert_eq!(role, OrgRole::Viewer);
    }

    #[test]
    fn test_user_response_from_user() {
        let user = User {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "secret_hash".to_string(),
            name: "Test User".to_string(),
            role: OrgRole::Admin,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = UserResponse::from(user.clone());

        assert_eq!(response.id, user.id);
        assert_eq!(response.org_id, user.org_id);
        assert_eq!(response.email, user.email);
        assert_eq!(response.name, user.name);
        assert_eq!(response.role, user.role);
        // password_hash should NOT be in response (checked by type system)
    }

    #[test]
    fn test_org_role_serialization() {
        assert_eq!(
            serde_json::to_string(&OrgRole::Admin).unwrap(),
            r#""admin""#
        );
        assert_eq!(
            serde_json::to_string(&OrgRole::Editor).unwrap(),
            r#""editor""#
        );
        assert_eq!(
            serde_json::to_string(&OrgRole::Viewer).unwrap(),
            r#""viewer""#
        );
    }

    #[test]
    fn test_org_role_deserialization() {
        assert_eq!(
            serde_json::from_str::<OrgRole>(r#""admin""#).unwrap(),
            OrgRole::Admin
        );
        assert_eq!(
            serde_json::from_str::<OrgRole>(r#""editor""#).unwrap(),
            OrgRole::Editor
        );
        assert_eq!(
            serde_json::from_str::<OrgRole>(r#""viewer""#).unwrap(),
            OrgRole::Viewer
        );
    }
}

mod datasource_tests {
    use crate::models::{DatasourceType, CreateDatasourceRequest, DatasourceResponse, Datasource};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_datasource_type_serialization() {
        assert_eq!(
            serde_json::to_string(&DatasourceType::Postgres).unwrap(),
            r#""postgres""#
        );
    }

    #[test]
    fn test_create_datasource_request_default_type() {
        let json = r#"{
            "name": "My DB",
            "connection_string": "postgres://localhost/db"
        }"#;
        
        let req: CreateDatasourceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ds_type, DatasourceType::Postgres);
    }

    #[test]
    fn test_datasource_response_excludes_connection_string() {
        let ds = Datasource {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            name: "Test DS".to_string(),
            ds_type: DatasourceType::Postgres,
            connection_string_encrypted: "secret_connection".to_string(),
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = DatasourceResponse::from(ds);
        let json = serde_json::to_string(&response).unwrap();
        
        // Connection string should NOT be in the serialized output
        assert!(!json.contains("secret_connection"));
        assert!(!json.contains("connection_string"));
    }
}

mod query_tests {
    use crate::models::{ParamType, ParamDef, CreateQueryRequest, QueryResponse, Query};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_param_type_serialization() {
        assert_eq!(serde_json::to_string(&ParamType::String).unwrap(), r#""string""#);
        assert_eq!(serde_json::to_string(&ParamType::Number).unwrap(), r#""number""#);
        assert_eq!(serde_json::to_string(&ParamType::Boolean).unwrap(), r#""boolean""#);
        assert_eq!(serde_json::to_string(&ParamType::Date).unwrap(), r#""date""#);
        assert_eq!(serde_json::to_string(&ParamType::DateTime).unwrap(), r#""datetime""#);
    }

    #[test]
    fn test_param_def_with_default() {
        let param = ParamDef {
            name: "limit".to_string(),
            param_type: ParamType::Number,
            default: Some(serde_json::json!(100)),
            required: false,
        };

        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains("limit"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_create_query_request_defaults() {
        let json = r#"{
            "datasource_id": "00000000-0000-0000-0000-000000000001",
            "name": "My Query",
            "sql": "SELECT 1"
        }"#;
        
        let req: CreateQueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.timeout_seconds, 30);
        assert_eq!(req.max_rows, 10000);
        assert!(req.parameters.is_empty());
    }

    #[test]
    fn test_query_response_parses_parameters() {
        let query = Query {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            datasource_id: Uuid::new_v4(),
            name: "Test Query".to_string(),
            description: Some("A test".to_string()),
            sql: "SELECT $1".to_string(),
            parameters: serde_json::json!([
                {"name": "id", "param_type": "number", "required": true}
            ]),
            timeout_seconds: 30,
            max_rows: 1000,
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = QueryResponse::from(query);
        assert_eq!(response.parameters.len(), 1);
        assert_eq!(response.parameters[0].name, "id");
    }
}

mod run_tests {
    use crate::models::{RunStatus, ExecuteAdHocRequest, ColumnDef, RunResultResponse, RunResult};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_run_status_serialization() {
        assert_eq!(serde_json::to_string(&RunStatus::Queued).unwrap(), r#""queued""#);
        assert_eq!(serde_json::to_string(&RunStatus::Running).unwrap(), r#""running""#);
        assert_eq!(serde_json::to_string(&RunStatus::Completed).unwrap(), r#""completed""#);
        assert_eq!(serde_json::to_string(&RunStatus::Failed).unwrap(), r#""failed""#);
        assert_eq!(serde_json::to_string(&RunStatus::Cancelled).unwrap(), r#""cancelled""#);
        assert_eq!(serde_json::to_string(&RunStatus::Timeout).unwrap(), r#""timeout""#);
    }

    #[test]
    fn test_execute_adhoc_request_defaults() {
        let json = r#"{
            "datasource_id": "00000000-0000-0000-0000-000000000001",
            "sql": "SELECT 1"
        }"#;
        
        let req: ExecuteAdHocRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.timeout_seconds, 30);
        assert_eq!(req.max_rows, 10000);
    }

    #[test]
    fn test_column_def_serialization() {
        let col = ColumnDef {
            name: "id".to_string(),
            data_type: "INT8".to_string(),
        };

        let json = serde_json::to_string(&col).unwrap();
        assert!(json.contains("id"));
        assert!(json.contains("INT8"));
    }

    #[test]
    fn test_run_result_response_parses_data() {
        let result = RunResult {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            columns: serde_json::json!([
                {"name": "id", "data_type": "INT8"},
                {"name": "name", "data_type": "TEXT"}
            ]),
            rows: serde_json::json!([
                [1, "Alice"],
                [2, "Bob"]
            ]),
            row_count: 2,
            byte_count: 100,
            execution_time_ms: 50,
            created_at: Utc::now(),
            expires_at: None,
        };

        let response = RunResultResponse::from(result);
        assert_eq!(response.row_count, 2);
        assert_eq!(response.columns.len(), 2);
        assert_eq!(response.rows.len(), 2);
        assert_eq!(response.execution_time_ms, 50);
    }
}

mod visualization_tests {
    use crate::models::{ChartType, CreateVisualizationRequest};

    #[test]
    fn test_chart_type_serialization() {
        assert_eq!(serde_json::to_string(&ChartType::Table).unwrap(), r#""table""#);
        assert_eq!(serde_json::to_string(&ChartType::Line).unwrap(), r#""line""#);
        assert_eq!(serde_json::to_string(&ChartType::Bar).unwrap(), r#""bar""#);
        assert_eq!(serde_json::to_string(&ChartType::SingleStat).unwrap(), r#""single_stat""#);
    }

    #[test]
    fn test_create_visualization_request() {
        let json = r#"{
            "query_id": "00000000-0000-0000-0000-000000000001",
            "name": "My Chart",
            "chart_type": "line"
        }"#;
        
        let req: CreateVisualizationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.chart_type, ChartType::Line);
        assert!(req.config.is_null() || req.config == serde_json::json!({}));
    }
}

mod dashboard_tests {
    use crate::models::{CreateTileRequest, TileResponse, Tile};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_create_tile_request_defaults() {
        let json = r#"{
            "visualization_id": "00000000-0000-0000-0000-000000000001"
        }"#;
        
        let req: CreateTileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.pos_x, 0);
        assert_eq!(req.pos_y, 0);
        assert_eq!(req.width, 4);
        assert_eq!(req.height, 4);
    }

    #[test]
    fn test_tile_response_from_tile() {
        let tile = Tile {
            id: Uuid::new_v4(),
            dashboard_id: Uuid::new_v4(),
            visualization_id: Uuid::new_v4(),
            title: Some("My Tile".to_string()),
            pos_x: 0,
            pos_y: 0,
            width: 6,
            height: 4,
            parameter_bindings: serde_json::json!({"date": "dashboard.date"}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = TileResponse::from(tile.clone());
        assert_eq!(response.id, tile.id);
        assert_eq!(response.width, 6);
        assert_eq!(response.parameter_bindings, tile.parameter_bindings);
    }
}

mod schedule_tests {
    use crate::models::{CreateScheduleRequest, ScheduleResponse, Schedule};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_create_schedule_request_defaults() {
        let json = r#"{
            "query_id": "00000000-0000-0000-0000-000000000001",
            "name": "Daily Refresh",
            "cron_expression": "0 0 * * *"
        }"#;
        
        let req: CreateScheduleRequest = serde_json::from_str(json).unwrap();
        assert!(req.enabled);
        assert!(req.parameters.is_null() || req.parameters == serde_json::json!({}));
    }

    #[test]
    fn test_schedule_response() {
        let schedule = Schedule {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            query_id: Uuid::new_v4(),
            name: "Hourly".to_string(),
            cron_expression: "0 * * * *".to_string(),
            parameters: serde_json::json!({}),
            enabled: true,
            last_run_at: None,
            next_run_at: Some(Utc::now()),
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = ScheduleResponse::from(schedule.clone());
        assert_eq!(response.cron_expression, "0 * * * *");
        assert!(response.enabled);
    }
}

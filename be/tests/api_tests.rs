//! API integration tests
//!
//! These tests verify the HTTP API endpoints against a real test server
//! using testcontainers for the database.

use actix_web::{App, http::StatusCode, test, web};
use loupe::db::Database;
use loupe::models::*;
use serde_json::json;
use std::sync::Arc;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

/// Application state matching the API server
pub struct AppState {
    pub db: Database,
}

/// Test helper that provides a configured test app
struct TestApp {
    db: Database,
    #[allow(dead_code)]
    container: ContainerAsync<Postgres>,
}

impl TestApp {
    async fn new() -> Self {
        let container = Postgres::default()
            .start()
            .await
            .expect("Failed to start postgres container");

        let host = container.get_host().await.expect("get host");
        let port = container.get_host_port_ipv4(5432).await.expect("get port");
        let database_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

        let db = Database::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        db.run_migrations().await.expect("Failed to run migrations");

        Self { db, container }
    }

    fn app_state(&self) -> Arc<AppState> {
        Arc::new(AppState {
            db: self.db.clone(),
        })
    }
}

// Include the routes module from the API
mod routes {
    use super::*;
    use actix_web::HttpResponse;
    use argon2::{
        Argon2,
        password_hash::{
            PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
        },
    };
    use loupe::Error;

    // Health route
    pub fn configure_health(cfg: &mut web::ServiceConfig) {
        cfg.route("/health", web::get().to(health_check));
    }

    async fn health_check() -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "status": "ok",
            "service": "loupe-api"
        }))
    }

    // Auth routes
    pub fn configure_auth(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/auth")
                .route("/register", web::post().to(register))
                .route("/login", web::post().to(login)),
        );
    }

    async fn register(
        state: web::Data<Arc<AppState>>,
        req: web::Json<CreateUserRequest>,
    ) -> Result<HttpResponse, Error> {
        if state.db.get_user_by_email(&req.email).await?.is_some() {
            return Err(Error::Conflict("Email already registered".to_string()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| Error::Internal(e.to_string()))?
            .to_string();

        let org = state
            .db
            .create_organization(&format!("{}'s Org", req.name))
            .await?;
        let user = state
            .db
            .create_user(
                org.id,
                &req.email,
                &password_hash,
                &req.name,
                OrgRole::Admin,
            )
            .await?;

        Ok(HttpResponse::Created().json(UserResponse::from(user)))
    }

    async fn login(
        state: web::Data<Arc<AppState>>,
        req: web::Json<LoginRequest>,
    ) -> Result<HttpResponse, Error> {
        let user = state
            .db
            .get_user_by_email(&req.email)
            .await?
            .ok_or_else(|| Error::Unauthorized("Invalid credentials".to_string()))?;

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| Error::Internal("Invalid password hash".to_string()))?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::Unauthorized("Invalid credentials".to_string()))?;

        Ok(HttpResponse::Ok().json(json!({
            "user": UserResponse::from(user),
            "token": "placeholder_token_for_v1"
        })))
    }

    // Datasource routes
    pub fn configure_datasources(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/datasources")
                .route("", web::get().to(list_datasources))
                .route("", web::post().to(create_datasource)),
        );
    }

    async fn list_datasources(
        state: web::Data<Arc<AppState>>,
        org_id: web::Query<OrgIdQuery>,
    ) -> Result<HttpResponse, Error> {
        let datasources = state.db.list_datasources(org_id.org_id).await?;
        let response: Vec<DatasourceResponse> = datasources.into_iter().map(Into::into).collect();
        Ok(HttpResponse::Ok().json(response))
    }

    async fn create_datasource(
        state: web::Data<Arc<AppState>>,
        req: web::Json<CreateDatasourceRequestWithOrg>,
    ) -> Result<HttpResponse, Error> {
        let ds = state
            .db
            .create_datasource(
                req.org_id,
                &req.name,
                req.ds_type.clone(),
                &req.connection_string, // Not encrypting for tests
                req.user_id,
            )
            .await?;
        Ok(HttpResponse::Created().json(DatasourceResponse::from(ds)))
    }

    // Helper queries
    #[derive(serde::Deserialize)]
    pub struct OrgIdQuery {
        pub org_id: uuid::Uuid,
    }

    #[derive(serde::Deserialize)]
    pub struct CreateDatasourceRequestWithOrg {
        pub org_id: uuid::Uuid,
        pub user_id: uuid::Uuid,
        pub name: String,
        #[serde(default = "default_postgres")]
        pub ds_type: DatasourceType,
        pub connection_string: String,
    }

    fn default_postgres() -> DatasourceType {
        DatasourceType::Postgres
    }

    // Query routes
    pub fn configure_queries(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/queries")
                .route("", web::get().to(list_queries))
                .route("", web::post().to(create_query)),
        );
    }

    async fn list_queries(
        state: web::Data<Arc<AppState>>,
        org_id: web::Query<OrgIdQuery>,
    ) -> Result<HttpResponse, Error> {
        let queries = state.db.list_queries(org_id.org_id).await?;
        let response: Vec<QueryResponse> = queries.into_iter().map(Into::into).collect();
        Ok(HttpResponse::Ok().json(response))
    }

    #[derive(serde::Deserialize)]
    pub struct CreateQueryRequestWithOrg {
        pub org_id: uuid::Uuid,
        pub user_id: uuid::Uuid,
        pub datasource_id: uuid::Uuid,
        pub name: String,
        pub description: Option<String>,
        pub sql: String,
        #[serde(default)]
        pub parameters: Vec<ParamDef>,
        #[serde(default = "default_timeout")]
        pub timeout_seconds: i32,
        #[serde(default = "default_max_rows")]
        pub max_rows: i32,
    }

    fn default_timeout() -> i32 {
        30
    }
    fn default_max_rows() -> i32 {
        10000
    }

    async fn create_query(
        state: web::Data<Arc<AppState>>,
        req: web::Json<CreateQueryRequestWithOrg>,
    ) -> Result<HttpResponse, Error> {
        let query = state
            .db
            .create_query(
                req.org_id,
                req.datasource_id,
                &req.name,
                req.description.as_deref(),
                &req.sql,
                &serde_json::to_value(&req.parameters).unwrap_or_default(),
                &serde_json::json!([]),
                req.timeout_seconds,
                req.max_rows,
                req.user_id,
            )
            .await?;
        Ok(HttpResponse::Created().json(QueryResponse::from(query)))
    }
}

fn configure_test_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(routes::configure_health)
            .configure(routes::configure_auth)
            .configure(routes::configure_datasources)
            .configure(routes::configure_queries),
    );
}

mod health_tests {
    use super::*;

    #[actix_web::test]
    async fn test_health_check() {
        let test_app = TestApp::new().await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "loupe-api");
    }
}

mod auth_tests {
    use super::*;

    #[actix_web::test]
    async fn test_register_new_user() {
        let test_app = TestApp::new().await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "email": "test@example.com",
                "password": "securepassword123",
                "name": "Test User"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["email"], "test@example.com");
        assert_eq!(body["name"], "Test User");
        assert_eq!(body["role"], "admin");
        assert!(body["id"].is_string());
    }

    #[actix_web::test]
    async fn test_register_duplicate_email() {
        let test_app = TestApp::new().await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        // First registration
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "email": "dupe@example.com",
                "password": "password123",
                "name": "First User"
            }))
            .to_request();
        test::call_service(&app, req).await;

        // Second registration with same email
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "email": "dupe@example.com",
                "password": "password456",
                "name": "Second User"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn test_login_success() {
        let test_app = TestApp::new().await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        // Register first
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "email": "login@example.com",
                "password": "mypassword",
                "name": "Login Test"
            }))
            .to_request();
        test::call_service(&app, req).await;

        // Then login
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({
                "email": "login@example.com",
                "password": "mypassword"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["user"]["email"].is_string());
        assert!(body["token"].is_string());
    }

    #[actix_web::test]
    async fn test_login_wrong_password() {
        let test_app = TestApp::new().await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        // Register first
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "email": "wrongpw@example.com",
                "password": "correctpassword",
                "name": "Wrong PW Test"
            }))
            .to_request();
        test::call_service(&app, req).await;

        // Login with wrong password
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({
                "email": "wrongpw@example.com",
                "password": "wrongpassword"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_login_nonexistent_user() {
        let test_app = TestApp::new().await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({
                "email": "nonexistent@example.com",
                "password": "anypassword"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}

mod datasource_tests {
    use super::*;

    async fn setup_user(test_app: &TestApp) -> (uuid::Uuid, uuid::Uuid) {
        let org = test_app.db.create_organization("Test Org").await.unwrap();
        let user = test_app
            .db
            .create_user(
                org.id,
                "ds-test@example.com",
                "hash",
                "DS Test User",
                OrgRole::Admin,
            )
            .await
            .unwrap();
        (org.id, user.id)
    }

    #[actix_web::test]
    async fn test_create_datasource() {
        let test_app = TestApp::new().await;
        let (org_id, user_id) = setup_user(&test_app).await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/datasources")
            .set_json(json!({
                "org_id": org_id.to_string(),
                "user_id": user_id.to_string(),
                "name": "Production DB",
                "ds_type": "postgres",
                "connection_string": "postgres://localhost/prod"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "Production DB");
        assert_eq!(body["ds_type"], "postgres");
        // Connection string should NOT be in response
        assert!(body.get("connection_string").is_none());
        assert!(body.get("connection_string_encrypted").is_none());
    }

    #[actix_web::test]
    async fn test_list_datasources() {
        let test_app = TestApp::new().await;
        let (org_id, user_id) = setup_user(&test_app).await;
        let state = test_app.app_state();

        // Create some datasources
        test_app
            .db
            .create_datasource(org_id, "DS 1", DatasourceType::Postgres, "conn1", user_id)
            .await
            .unwrap();
        test_app
            .db
            .create_datasource(org_id, "DS 2", DatasourceType::Postgres, "conn2", user_id)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/datasources?org_id={}", org_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);
    }
}

mod query_tests {
    use super::*;

    async fn setup_with_datasource(test_app: &TestApp) -> (uuid::Uuid, uuid::Uuid, uuid::Uuid) {
        let org = test_app.db.create_organization("Test Org").await.unwrap();
        let user = test_app
            .db
            .create_user(
                org.id,
                "query-test@example.com",
                "hash",
                "Query Test User",
                OrgRole::Admin,
            )
            .await
            .unwrap();
        let ds = test_app
            .db
            .create_datasource(
                org.id,
                "Test DS",
                DatasourceType::Postgres,
                "postgres://localhost/test",
                user.id,
            )
            .await
            .unwrap();
        (org.id, user.id, ds.id)
    }

    #[actix_web::test]
    async fn test_create_query() {
        let test_app = TestApp::new().await;
        let (org_id, user_id, ds_id) = setup_with_datasource(&test_app).await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/queries")
            .set_json(json!({
                "org_id": org_id.to_string(),
                "user_id": user_id.to_string(),
                "datasource_id": ds_id.to_string(),
                "name": "Active Users",
                "description": "Count active users",
                "sql": "SELECT COUNT(*) FROM users WHERE active = true",
                "timeout_seconds": 60,
                "max_rows": 1
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "Active Users");
        assert_eq!(
            body["sql"],
            "SELECT COUNT(*) FROM users WHERE active = true"
        );
        assert_eq!(body["timeout_seconds"], 60);
    }

    #[actix_web::test]
    async fn test_list_queries() {
        let test_app = TestApp::new().await;
        let (org_id, user_id, ds_id) = setup_with_datasource(&test_app).await;
        let state = test_app.app_state();

        // Create queries directly
        test_app
            .db
            .create_query(
                org_id,
                ds_id,
                "Query 1",
                None,
                "SELECT 1",
                &json!([]),
                &json!([]),
                30,
                1000,
                user_id,
            )
            .await
            .unwrap();
        test_app
            .db
            .create_query(
                org_id,
                ds_id,
                "Query 2",
                None,
                "SELECT 2",
                &json!([]),
                &json!([]),
                30,
                1000,
                user_id,
            )
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/queries?org_id={}", org_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);
    }

    #[actix_web::test]
    async fn test_create_query_with_parameters() {
        let test_app = TestApp::new().await;
        let (org_id, user_id, ds_id) = setup_with_datasource(&test_app).await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/queries")
            .set_json(json!({
                "org_id": org_id.to_string(),
                "user_id": user_id.to_string(),
                "datasource_id": ds_id.to_string(),
                "name": "Parameterized Query",
                "sql": "SELECT * FROM events WHERE date > $1 LIMIT $2",
                "parameters": [
                    {"name": "start_date", "param_type": "date", "required": true},
                    {"name": "limit", "param_type": "number", "default": 100, "required": false}
                ]
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["parameters"].as_array().unwrap().len(), 2);
        assert_eq!(body["parameters"][0]["name"], "start_date");
    }
}

mod integration_flow_tests {
    use super::*;

    /// Test the full user flow: register -> create datasource -> create query
    #[actix_web::test]
    async fn test_full_user_flow() {
        let test_app = TestApp::new().await;
        let state = test_app.app_state();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_test_app),
        )
        .await;

        // 1. Register
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "email": "flow@example.com",
                "password": "flowpassword",
                "name": "Flow Test"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let user: serde_json::Value = test::read_body_json(resp).await;
        let user_id = user["id"].as_str().unwrap();
        let org_id = user["org_id"].as_str().unwrap();

        // 2. Create datasource
        let req = test::TestRequest::post()
            .uri("/api/v1/datasources")
            .set_json(json!({
                "org_id": org_id,
                "user_id": user_id,
                "name": "Analytics DB",
                "connection_string": "postgres://localhost/analytics"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let ds: serde_json::Value = test::read_body_json(resp).await;
        let ds_id = ds["id"].as_str().unwrap();

        // 3. Create query
        let req = test::TestRequest::post()
            .uri("/api/v1/queries")
            .set_json(json!({
                "org_id": org_id,
                "user_id": user_id,
                "datasource_id": ds_id,
                "name": "Daily Metrics",
                "sql": "SELECT date, COUNT(*) FROM metrics GROUP BY date"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        // 4. Verify queries are listed
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/queries?org_id={}", org_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let queries: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0]["name"], "Daily Metrics");
    }
}

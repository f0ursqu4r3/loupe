//! Test fixtures for generating test data

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use fake::{Fake, faker::internet::en::SafeEmail, faker::name::en::Name};
use loupe::Database;
use loupe::models::*;
use uuid::Uuid;

/// Fixture for creating test organizations
pub struct OrgFixture;

impl OrgFixture {
    pub async fn create(db: &Database) -> Organization {
        let name: String = fake::faker::company::en::CompanyName().fake();
        db.create_organization(&name).await.unwrap()
    }

    pub async fn create_with_name(db: &Database, name: &str) -> Organization {
        db.create_organization(name).await.unwrap()
    }
}

/// Fixture for creating test users
pub struct UserFixture;

impl UserFixture {
    pub async fn create(db: &Database, org_id: Uuid) -> User {
        let email: String = SafeEmail().fake();
        let name: String = Name().fake();
        let password_hash = hash_password("testpassword123");

        db.create_user(org_id, &email, &password_hash, &name, OrgRole::Editor)
            .await
            .unwrap()
    }

    pub async fn create_admin(db: &Database, org_id: Uuid) -> User {
        let email: String = SafeEmail().fake();
        let name: String = Name().fake();
        let password_hash = hash_password("testpassword123");

        db.create_user(org_id, &email, &password_hash, &name, OrgRole::Admin)
            .await
            .unwrap()
    }

    pub async fn create_with_email(db: &Database, org_id: Uuid, email: &str) -> User {
        let name: String = Name().fake();
        let password_hash = hash_password("testpassword123");

        db.create_user(org_id, email, &password_hash, &name, OrgRole::Editor)
            .await
            .unwrap()
    }
}

/// Fixture for creating test datasources
pub struct DatasourceFixture;

impl DatasourceFixture {
    pub async fn create(db: &Database, org_id: Uuid, created_by: Uuid) -> Datasource {
        let name = format!(
            "Test Datasource {}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );
        let conn_string = "postgres://test:test@localhost:5432/testdb";

        db.create_datasource(
            org_id,
            &name,
            DatasourceType::Postgres,
            conn_string,
            created_by,
        )
        .await
        .unwrap()
    }

    pub async fn create_with_connection(
        db: &Database,
        org_id: Uuid,
        created_by: Uuid,
        connection_string: &str,
    ) -> Datasource {
        let name = format!(
            "Test Datasource {}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        db.create_datasource(
            org_id,
            &name,
            DatasourceType::Postgres,
            connection_string,
            created_by,
        )
        .await
        .unwrap()
    }
}

/// Fixture for creating test queries
pub struct QueryFixture;

impl QueryFixture {
    pub async fn create(
        db: &Database,
        org_id: Uuid,
        datasource_id: Uuid,
        created_by: Uuid,
    ) -> Query {
        let name = format!(
            "Test Query {}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );
        let sql = "SELECT 1 as value";
        let parameters = serde_json::json!([]);

        db.create_query(
            org_id,
            datasource_id,
            &name,
            Some("A test query"),
            sql,
            &parameters,
            &serde_json::json!([]),
            30,
            10000,
            created_by,
        )
        .await
        .unwrap()
    }

    pub async fn create_with_sql(
        db: &Database,
        org_id: Uuid,
        datasource_id: Uuid,
        created_by: Uuid,
        sql: &str,
    ) -> Query {
        let name = format!(
            "Test Query {}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );
        let parameters = serde_json::json!([]);

        db.create_query(
            org_id,
            datasource_id,
            &name,
            None,
            sql,
            &parameters,
            &serde_json::json!([]),
            30,
            10000,
            created_by,
        )
        .await
        .unwrap()
    }
}

/// Fixture for creating test runs
pub struct RunFixture;

impl RunFixture {
    pub async fn create(
        db: &Database,
        org_id: Uuid,
        query_id: Uuid,
        datasource_id: Uuid,
        created_by: Uuid,
    ) -> Run {
        db.create_run(
            org_id,
            query_id,
            datasource_id,
            "SELECT 1 as value",
            &serde_json::json!({}),
            30,
            10000,
            created_by,
        )
        .await
        .unwrap()
    }
}

/// Fixture for creating test visualizations
pub struct VisualizationFixture;

impl VisualizationFixture {
    pub async fn create(
        db: &Database,
        org_id: Uuid,
        query_id: Uuid,
        created_by: Uuid,
    ) -> Visualization {
        let name = format!(
            "Test Viz {}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );
        let config = serde_json::json!({
            "x_axis": "time",
            "y_axis": "value"
        });

        db.create_visualization(
            org_id,
            query_id,
            &name,
            ChartType::Line,
            &config,
            &serde_json::json!([]),
            created_by,
        )
        .await
        .unwrap()
    }
}

/// Fixture for creating test dashboards
pub struct DashboardFixture;

impl DashboardFixture {
    pub async fn create(db: &Database, org_id: Uuid, created_by: Uuid) -> Dashboard {
        let name = format!(
            "Test Dashboard {}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        db.create_dashboard(
            org_id,
            &name,
            Some("Test description"),
            &serde_json::json!([]),
            &serde_json::json!([]),
            created_by,
        )
        .await
        .unwrap()
    }
}

/// Fixture for creating test schedules
pub struct ScheduleFixture;

impl ScheduleFixture {
    pub async fn create(db: &Database, org_id: Uuid, query_id: Uuid, created_by: Uuid) -> Schedule {
        let name = format!(
            "Test Schedule {}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        db.create_schedule(
            org_id,
            query_id,
            &name,
            "0 */15 * * *", // Every 15 minutes
            &serde_json::json!({}),
            &serde_json::json!([]),
            true,
            created_by,
        )
        .await
        .unwrap()
    }
}

/// Helper to hash a password for tests
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

/// A complete test setup with org, user, datasource, and query
pub struct FullTestSetup {
    pub org: Organization,
    pub user: User,
    pub datasource: Datasource,
    pub query: Query,
}

impl FullTestSetup {
    pub async fn create(db: &Database) -> Self {
        let org = OrgFixture::create(db).await;
        let user = UserFixture::create(db, org.id).await;
        let datasource = DatasourceFixture::create(db, org.id, user.id).await;
        let query = QueryFixture::create(db, org.id, datasource.id, user.id).await;

        Self {
            org,
            user,
            datasource,
            query,
        }
    }
}

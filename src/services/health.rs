use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::database::connection::DatabaseManager;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub database: DatabaseHealth,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: String,
    pub connection_pool_size: u32,
    pub response_time_ms: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub start_time: DateTime<Utc>,
}

pub struct HealthService {
    pub router: Router,
}

impl HealthService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        let state = AppState {
            db,
            start_time: Utc::now(),
        };

        let router = Router::new()
            .route("/health", get(health_check))
            .route("/health/ready", get(readiness_check))
            .route("/health/live", get(liveness_check))
            .with_state(state);

        Self { router }
    }
}

async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let start = std::time::Instant::now();
    
    // Test database connectivity
    let db_status = match test_database_connection(&state.db).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };
    
    let response_time_ms = start.elapsed().as_millis() as u64;
    let uptime = Utc::now()
        .signed_duration_since(state.start_time)
        .num_seconds() as u64;

    let health_response = HealthResponse {
        status: if db_status == "healthy" { "healthy".to_string() } else { "unhealthy".to_string() },
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: DatabaseHealth {
            status: db_status.to_string(),
            connection_pool_size: state.db.pool.size(),
            response_time_ms,
        },
        uptime_seconds: uptime,
    };

    if health_response.status == "healthy" {
        Ok(Json(health_response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn readiness_check(State(state): State<AppState>) -> Result<Json<&'static str>, StatusCode> {
    // Check if the service is ready to accept traffic
    match test_database_connection(&state.db).await {
        Ok(_) => Ok(Json("ready")),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

async fn liveness_check() -> Json<&'static str> {
    // Simple liveness check - if this endpoint responds, the service is alive
    Json("alive")
}

async fn test_database_connection(db: &DatabaseManager) -> Result<(), sqlx::Error> {
    // Test database connectivity with a simple query
    sqlx::query("SELECT 1")
        .fetch_one(&db.pool)
        .await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use tempfile::TempDir;

    async fn create_test_health_service() -> (HealthService, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite://{}", db_path.display());
        
        let db = Arc::new(
            DatabaseManager::new(&db_url)
                .await
                .expect("Failed to create test database")
        );
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&db.pool)
            .await
            .expect("Failed to run migrations");
        
        (HealthService::new(db), temp_dir)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let (health_service, _temp_dir) = create_test_health_service().await;
        let server = TestServer::new(health_service.router).expect("Failed to create test server");

        let response = server.get("/health").await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let health_response: HealthResponse = response.json();
        assert_eq!(health_response.status, "healthy");
        assert_eq!(health_response.database.status, "healthy");
        assert_eq!(health_response.version, env!("CARGO_PKG_VERSION"));
        assert!(health_response.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn test_readiness_endpoint() {
        let (health_service, _temp_dir) = create_test_health_service().await;
        let server = TestServer::new(health_service.router).expect("Failed to create test server");

        let response = server.get("/health/ready").await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let ready_response: String = response.json();
        assert_eq!(ready_response, "ready");
    }

    #[tokio::test]
    async fn test_liveness_endpoint() {
        let (health_service, _temp_dir) = create_test_health_service().await;
        let server = TestServer::new(health_service.router).expect("Failed to create test server");

        let response = server.get("/health/live").await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let alive_response: String = response.json();
        assert_eq!(alive_response, "alive");
    }
}
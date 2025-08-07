//! 💚 Health Check Handler

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::AppState;
use crate::models::ApiResult;

/// Response do health check
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Status da aplicação
    pub status: String,
    /// Nome da aplicação
    pub app_name: String,
    /// Versão da aplicação
    pub version: String,
    /// Ambiente
    pub environment: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
pub async fn health_check(
    State(state): State<AppState>,
) -> ApiResult<Json<HealthResponse>> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        app_name: state.config.app.name.clone(),
        version: state.config.app.version.clone(),
        environment: state.config.app.environment.clone(),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

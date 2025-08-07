//! 📊 System Status Handlers

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::api::AppState;
use crate::models::ApiResult;

/// Response do status do sistema
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SystemStatusResponse {
    /// Status geral do sistema
    pub status: String,
    /// Status do FreeSWITCH
    pub freeswitch_connected: bool,
    /// Informações da aplicação
    pub application: ApplicationInfo,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Informações da aplicação
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplicationInfo {
    /// Nome da aplicação
    pub name: String,
    /// Versão
    pub version: String,
    /// Ambiente
    pub environment: String,
    /// Configuração do servidor
    pub server: ServerInfo,
}

/// Informações do servidor
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServerInfo {
    /// Host
    pub host: String,
    /// Porta
    pub port: u16,
}

/// Response das estatísticas
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatisticsResponse {
    /// Estatísticas de chamadas
    pub call_statistics: HashMap<String, usize>,
    /// Uptime em segundos
    pub uptime_seconds: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Obtém status do sistema
#[utoipa::path(
    get,
    path = "/api/v1/status",
    tag = "status",
    responses(
        (status = 200, description = "System status", body = SystemStatusResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_status(
    State(state): State<AppState>,
) -> ApiResult<Json<SystemStatusResponse>> {
    // Verificar se FreeSWITCH está conectado
    let freeswitch_connected = true; // Implementação simplificada
    
    let response = SystemStatusResponse {
        status: if freeswitch_connected { "healthy" } else { "degraded" }.to_string(),
        freeswitch_connected,
        application: ApplicationInfo {
            name: state.config.app.name.clone(),
            version: state.config.app.version.clone(),
            environment: state.config.app.environment.clone(),
            server: ServerInfo {
                host: state.config.server.host.clone(),
                port: state.config.server.port,
            },
        },
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

/// Obtém estatísticas do sistema
#[utoipa::path(
    get,
    path = "/api/v1/statistics",
    tag = "status",
    responses(
        (status = 200, description = "System statistics", body = StatisticsResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_statistics(
    State(state): State<AppState>,
) -> ApiResult<Json<StatisticsResponse>> {
    let call_statistics = state.call_manager.get_call_statistics().await;
    
    let response = StatisticsResponse {
        call_statistics,
        uptime_seconds: 0, // TODO: Implementar cálculo real de uptime
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

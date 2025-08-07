//! ❌ Error Model - Tratamento de erros da aplicação

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

/// Erros da API
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("FreeSWITCH connection error: {0}")]
    FreeSwitchConnection(String),
    
    #[error("Call not found: {0}")]
    CallNotFound(Uuid),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Too many requests: {0}")]
    TooManyRequests(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

/// Response de erro padronizada da API
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Código do erro
    pub error_code: String,
    /// Mensagem de erro
    pub message: String,
    /// Detalhes adicionais (opcional)
    pub details: Option<String>,
    /// Timestamp do erro
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ApiError {
    /// Converte para código HTTP
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::CallNotFound(_) | ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InvalidRequest(_) | ApiError::Validation(_) | ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::FreeSwitchConnection(_) | ApiError::ServiceUnavailable(_) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            ApiError::Internal(_) | ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Converte para código de erro
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::FreeSwitchConnection(_) => "FREESWITCH_CONNECTION_ERROR",
            ApiError::CallNotFound(_) => "CALL_NOT_FOUND",
            ApiError::InvalidRequest(_) => "INVALID_REQUEST",
            ApiError::Configuration(_) => "CONFIGURATION_ERROR",
            ApiError::Internal(_) => "INTERNAL_ERROR",
            ApiError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            ApiError::Validation(_) => "VALIDATION_ERROR",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::Forbidden(_) => "FORBIDDEN",
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::TooManyRequests(_) => "TOO_MANY_REQUESTS",
        }
    }

    /// Converte para response de erro
    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            error_code: self.error_code().to_string(),
            message: self.to_string(),
            details: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = self.to_error_response();
        
        tracing::error!(
            error_code = error_response.error_code,
            message = error_response.message,
            status = %status,
            "API error occurred"
        );
        
        (status, Json(error_response)).into_response()
    }
}

/// Conversões de erros externos
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl From<config::ConfigError> for ApiError {
    fn from(err: config::ConfigError) -> Self {
        ApiError::Configuration(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::InvalidRequest(format!("JSON parsing error: {}", err))
    }
}

/// Tipo de resultado padrão da aplicação
pub type ApiResult<T> = Result<T, ApiError>;

/// Alias para ErrorResponse para compatibilidade OpenAPI
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename = "ApiError")]
pub struct ApiErrorResponse {
    /// Código do erro
    pub error_code: String,
    /// Mensagem de erro
    pub message: String,
    /// Detalhes adicionais (opcional)
    pub details: Option<String>,
    /// Timestamp do erro
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
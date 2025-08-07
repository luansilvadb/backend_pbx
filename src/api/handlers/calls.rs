//! 📞 Call Management Handlers

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::models::{ApiResult, Call, CallListResponse, OriginateCallRequest};

/// Lista todas as chamadas ativas
#[utoipa::path(
    get,
    path = "/api/v1/calls",
    tag = "calls",
    responses(
        (status = 200, description = "List of active calls", body = CallListResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn list_calls(
    State(state): State<AppState>,
) -> ApiResult<Json<CallListResponse>> {
    let calls = state.call_manager.list_active_calls().await;
    let total = calls.len();

    let response = CallListResponse { calls, total };
    Ok(Json(response))
}

/// Obtém detalhes de uma chamada específica
#[utoipa::path(
    get,
    path = "/api/v1/calls/{id}",
    tag = "calls",
    params(
        ("id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call details", body = Call),
        (status = 404, description = "Call not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_call(
    State(state): State<AppState>,
    Path(call_id): Path<Uuid>,
) -> ApiResult<Json<Call>> {
    let call = state.call_manager.get_call(&call_id).await?;
    Ok(Json(call))
}

/// Origina uma nova chamada
#[utoipa::path(
    post,
    path = "/api/v1/calls",
    tag = "calls",
    request_body = OriginateCallRequest,
    responses(
        (status = 201, description = "Call originated successfully", body = Call),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
        (status = 503, description = "FreeSWITCH unavailable", body = ApiErrorResponse),
    )
)]
pub async fn originate_call(
    State(state): State<AppState>,
    Json(request): Json<OriginateCallRequest>,
) -> ApiResult<Json<Call>> {
    // Validação básica
    if request.from.is_empty() || request.to.is_empty() {
        return Err(crate::models::ApiError::Validation(
            "From and to fields are required".to_string(),
        ));
    }

    let call = state.call_manager.originate_call(request).await?;
    Ok(Json(call))
}

/// Encerra uma chamada
#[utoipa::path(
    post,
    path = "/api/v1/calls/{id}/hangup",
    tag = "calls",
    params(
        ("id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call hung up successfully"),
        (status = 404, description = "Call not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn hangup_call(
    State(state): State<AppState>,
    Path(call_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.call_manager.hangup_call(&call_id).await?;
    
    Ok(Json(serde_json::json!({
        "message": "Call hung up successfully",
        "call_id": call_id
    })))
}

//! 📞 SIP Users Handlers - Endpoints para gerenciamento de usuários SIP
//! 
//! Este módulo implementa os handlers HTTP para operações CRUD de usuários SIP,
//! incluindo criação, autenticação, registro e gerenciamento de extensões.

use crate::models::{
    ApiError, ApiResult,
    SipUserResponse, SipUserListResponse, CreateSipUserRequest, UpdateSipUserRequest,
    SipRegistrationInfo, SipUserUsageStats, SipUserStatus,
};
use crate::api::AppState;

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use tracing::{info, instrument};
use utoipa::IntoParams;
use uuid::Uuid;

/// Parâmetros de query para listagem de usuários SIP
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListSipUsersQuery {
    /// Página (default: 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Itens por página (default: 20, max: 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    /// Filtro por status (opcional)
    pub status: Option<SipUserStatus>,
    /// Busca por nome ou extensão (opcional)
    pub search: Option<String>,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

/// Request para alteração de senha
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    /// Nova senha
    pub new_password: String,
}

/// Request para alteração de status
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetUserStatusRequest {
    /// Novo status
    pub status: SipUserStatus,
}

// ==================== ENDPOINTS CRUD ====================

/// Lista usuários SIP de um tenant
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}/sip-users",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ListSipUsersQuery
    ),
    responses(
        (status = 200, description = "Lista de usuários SIP", body = SipUserListResponse),
        (status = 400, description = "Parâmetros inválidos", body = ApiErrorResponse),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state))]
pub async fn list_sip_users(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
    Query(params): Query<ListSipUsersQuery>,
) -> ApiResult<Json<SipUserListResponse>> {
    // Validar que tenant existe
    if app_state.tenant_service.get_tenant(&tenant_id).await?.is_none() {
        return Err(ApiError::NotFound("Tenant not found".to_string()));
    }

    // Validar parâmetros
    let page = params.page.max(1);
    let per_page = params.per_page.min(100).max(1);

    info!("Listing SIP users for tenant {}: page={}, per_page={}", tenant_id, page, per_page);

    let response = app_state.sip_user_service.list_users(&tenant_id, page, per_page).await?;
    
    Ok(Json(response))
}

/// Cria novo usuário SIP
#[utoipa::path(
    post,
    path = "/api/tenants/{tenant_id}/sip-users",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    request_body = CreateSipUserRequest,
    responses(
        (status = 201, description = "Usuário SIP criado com sucesso", body = SipUserResponse),
        (status = 400, description = "Dados inválidos", body = ApiErrorResponse),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 409, description = "Extensão já existe", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state, request))]
pub async fn create_sip_user(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<CreateSipUserRequest>,
) -> ApiResult<Json<SipUserResponse>> {
    // Validar que tenant existe
    if app_state.tenant_service.get_tenant(&tenant_id).await?.is_none() {
        return Err(ApiError::NotFound("Tenant not found".to_string()));
    }

    info!("Creating SIP user {} for tenant {}", request.extension, tenant_id);

    let user = app_state.sip_user_service.create_user(&tenant_id, request).await?;
    
    Ok(Json(user))
}

/// Obtém detalhes de um usuário SIP
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}/sip-users/{user_id}",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ("user_id" = Uuid, Path, description = "ID único do usuário SIP")
    ),
    responses(
        (status = 200, description = "Detalhes do usuário SIP", body = SipUserResponse),
        (status = 404, description = "Usuário não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state))]
pub async fn get_sip_user(
    State(app_state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, Uuid)>,
) -> ApiResult<Json<SipUserResponse>> {
    info!("Getting SIP user {} for tenant {}", user_id, tenant_id);

    match app_state.sip_user_service.get_user(&tenant_id, user_id).await? {
        Some(user) => Ok(Json(user)),
        None => Err(ApiError::NotFound("SIP user not found".to_string())),
    }
}

/// Obtém usuário SIP por extensão
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}/sip-users/extension/{extension}",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ("extension" = String, Path, description = "Número da extensão")
    ),
    responses(
        (status = 200, description = "Detalhes do usuário SIP", body = SipUserResponse),
        (status = 404, description = "Usuário não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state))]
pub async fn get_sip_user_by_extension(
    State(app_state): State<AppState>,
    Path((tenant_id, extension)): Path<(String, String)>,
) -> ApiResult<Json<SipUserResponse>> {
    info!("Getting SIP user by extension {} for tenant {}", extension, tenant_id);

    match app_state.sip_user_service.get_user_by_extension(&tenant_id, &extension).await? {
        Some(user) => Ok(Json(user)),
        None => Err(ApiError::NotFound("SIP user not found".to_string())),
    }
}

/// Atualiza usuário SIP
#[utoipa::path(
    put,
    path = "/api/tenants/{tenant_id}/sip-users/{user_id}",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ("user_id" = Uuid, Path, description = "ID único do usuário SIP")
    ),
    request_body = UpdateSipUserRequest,
    responses(
        (status = 200, description = "Usuário SIP atualizado com sucesso", body = SipUserResponse),
        (status = 400, description = "Dados inválidos", body = ApiErrorResponse),
        (status = 404, description = "Usuário não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state, request))]
pub async fn update_sip_user(
    State(app_state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, Uuid)>,
    Json(request): Json<UpdateSipUserRequest>,
) -> ApiResult<Json<SipUserResponse>> {
    info!("Updating SIP user {} for tenant {}", user_id, tenant_id);

    let user = app_state.sip_user_service.update_user(&tenant_id, user_id, request).await?;
    
    Ok(Json(user))
}

/// Remove usuário SIP
#[utoipa::path(
    delete,
    path = "/api/tenants/{tenant_id}/sip-users/{user_id}",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ("user_id" = Uuid, Path, description = "ID único do usuário SIP")
    ),
    responses(
        (status = 204, description = "Usuário SIP removido com sucesso"),
        (status = 404, description = "Usuário não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state))]
pub async fn delete_sip_user(
    State(app_state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, Uuid)>,
) -> ApiResult<()> {
    info!("Deleting SIP user {} for tenant {}", user_id, tenant_id);

    app_state.sip_user_service.delete_user(&tenant_id, user_id).await?;
    
    Ok(())
}

// ==================== ENDPOINTS ESPECIAIS ====================

/// Reset senha do usuário SIP
#[utoipa::path(
    post,
    path = "/api/tenants/{tenant_id}/sip-users/{user_id}/reset-password",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ("user_id" = Uuid, Path, description = "ID único do usuário SIP")
    ),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Senha alterada com sucesso"),
        (status = 400, description = "Senha inválida", body = ApiErrorResponse),
        (status = 404, description = "Usuário não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state, request))]
pub async fn reset_sip_user_password(
    State(app_state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, Uuid)>,
    Json(request): Json<ResetPasswordRequest>,
) -> ApiResult<()> {
    info!("Resetting password for SIP user {} in tenant {}", user_id, tenant_id);

    app_state.sip_user_service.reset_password(&tenant_id, user_id, &request.new_password).await?;
    
    Ok(())
}

/// Altera status do usuário SIP
#[utoipa::path(
    post,
    path = "/api/tenants/{tenant_id}/sip-users/{user_id}/status",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ("user_id" = Uuid, Path, description = "ID único do usuário SIP")
    ),
    request_body = SetUserStatusRequest,
    responses(
        (status = 200, description = "Status alterado com sucesso"),
        (status = 404, description = "Usuário não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state, request))]
pub async fn set_sip_user_status(
    State(app_state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, Uuid)>,
    Json(request): Json<SetUserStatusRequest>,
) -> ApiResult<()> {
    info!("Setting status {:?} for SIP user {} in tenant {}", request.status, user_id, tenant_id);

    app_state.sip_user_service.set_user_status(&tenant_id, user_id, request.status).await?;
    
    Ok(())
}

/// Lista registros SIP ativos
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}/sip-users/registrations",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    responses(
        (status = 200, description = "Lista de registros SIP ativos", body = Vec<SipRegistrationInfo>),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state))]
pub async fn list_sip_registrations(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<Vec<SipRegistrationInfo>>> {
    info!("Listing SIP registrations for tenant {}", tenant_id);

    // Validar que tenant existe
    if app_state.tenant_service.get_tenant(&tenant_id).await?.is_none() {
        return Err(ApiError::NotFound("Tenant not found".to_string()));
    }

    let registrations = app_state.sip_user_service.list_registrations(&tenant_id).await?;
    
    Ok(Json(registrations))
}

/// Obtém estatísticas de uso de um usuário SIP
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}/sip-users/{user_id}/usage",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ("user_id" = Uuid, Path, description = "ID único do usuário SIP")
    ),
    responses(
        (status = 200, description = "Estatísticas de uso", body = SipUserUsageStats),
        (status = 404, description = "Usuário não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-users"
)]
#[instrument(skip(app_state))]
pub async fn get_sip_user_usage(
    State(app_state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, Uuid)>,
) -> ApiResult<Json<SipUserUsageStats>> {
    info!("Getting usage statistics for SIP user {} in tenant {}", user_id, tenant_id);

    let usage = app_state.sip_user_service.get_user_usage(&tenant_id, user_id).await?;
    
    Ok(Json(usage))
}

// ==================== ENDPOINTS PARA FREESWITCH ====================

/// Endpoint para autenticação SIP (usado pelo FreeSWITCH)
/// Este endpoint é chamado pelo FreeSWITCH para validar credenciais
#[utoipa::path(
    post,
    path = "/api/sip/auth/{tenant_id}/{extension}",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant"),
        ("extension" = String, Path, description = "Número da extensão")
    ),
    request_body = SipAuthRequest,
    responses(
        (status = 200, description = "Autenticação bem-sucedida", body = SipAuthResponse),
        (status = 401, description = "Credenciais inválidas", body = ApiErrorResponse),
        (status = 404, description = "Usuário não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "sip-auth"
)]
#[instrument(skip(app_state, request))]
pub async fn authenticate_sip_user(
    State(app_state): State<AppState>,
    Path((tenant_id, extension)): Path<(String, String)>,
    Json(request): Json<SipAuthRequest>,
) -> ApiResult<Json<SipAuthResponse>> {
    info!("SIP authentication request for {}@{}", extension, tenant_id);

    let is_valid = app_state.sip_user_service
        .authenticate_user(&tenant_id, &extension, &request.password)
        .await?;

    if is_valid {
        // Atualizar registro
        app_state.sip_user_service
            .update_registration(&tenant_id, &extension, request.ip, request.user_agent)
            .await?;

        Ok(Json(SipAuthResponse {
            success: true,
            message: "Authentication successful".to_string(),
        }))
    } else {
        Err(ApiError::Unauthorized("Invalid credentials".to_string()))
    }
}

/// Request de autenticação SIP
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SipAuthRequest {
    /// Senha fornecida
    pub password: String,
    /// IP de origem (opcional)
    pub ip: Option<String>,
    /// User-Agent (opcional)
    pub user_agent: Option<String>,
}

/// Response de autenticação SIP
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SipAuthResponse {
    /// Se autenticação foi bem-sucedida
    pub success: bool,
    /// Mensagem de resultado
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    #[test]
    fn test_query_params_defaults() {
        let query = ListSipUsersQuery {
            page: 0, // Será corrigido para 1
            per_page: 200, // Será limitado a 100
            status: None,
            search: None,
        };

        assert_eq!(query.page.max(1), 1);
        assert_eq!(query.per_page.min(100).max(1), 100);
    }
}

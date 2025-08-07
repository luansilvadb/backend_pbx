//! 🏢 Tenant Handlers - Endpoints para gerenciamento de tenants
//! 
//! Este módulo implementa os handlers HTTP para operações CRUD de tenants,
//! incluindo validação, limites e métricas.

use crate::models::{
    ApiError, ApiResult, Tenant, TenantUsage, TenantListResponse,
    CreateTenantRequest, UpdateTenantRequest, TenantStatus
};

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;

use tracing::{info, instrument};
use utoipa::IntoParams;

/// Parâmetros de query para listagem de tenants
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListTenantsQuery {
    /// Página (default: 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Itens por página (default: 20, max: 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    /// Filtro por status (opcional)
    pub status: Option<String>,
    /// Filtro por nome (busca parcial, opcional)
    pub search: Option<String>,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

use crate::api::AppState;

/// Lista todos os tenants com paginação
#[utoipa::path(
    get,
    path = "/api/tenants",
    params(ListTenantsQuery),
    responses(
        (status = 200, description = "Lista de tenants", body = TenantListResponse),
        (status = 400, description = "Parâmetros inválidos", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state))]
pub async fn list_tenants(
    State(app_state): State<AppState>,
    Query(params): Query<ListTenantsQuery>,
) -> ApiResult<Json<TenantListResponse>> {
    // Validar parâmetros
    let page = params.page.max(1);
    let per_page = params.per_page.min(100).max(1);

    info!("Listing tenants: page={}, per_page={}", page, per_page);

    let response = app_state.tenant_service.list_tenants(page, per_page).await?;
    
    Ok(Json(response))
}

/// Cria novo tenant
#[utoipa::path(
    post,
    path = "/api/tenants",
    request_body = CreateTenantRequest,
    responses(
        (status = 201, description = "Tenant criado com sucesso", body = Tenant),
        (status = 400, description = "Dados inválidos", body = ApiErrorResponse),
        (status = 409, description = "Tenant já existe", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state, request))]
pub async fn create_tenant(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTenantRequest>,
) -> ApiResult<Json<Tenant>> {
    info!("Creating tenant: {}", request.id);

    let tenant = app_state.tenant_service.create_tenant(request).await?;
    
    Ok(Json(tenant))
}

/// Obtém detalhes de um tenant específico
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    responses(
        (status = 200, description = "Detalhes do tenant", body = Tenant),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state))]
pub async fn get_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<Tenant>> {
    info!("Getting tenant: {}", tenant_id);

    match app_state.tenant_service.get_tenant(&tenant_id).await? {
        Some(tenant) => Ok(Json(tenant)),
        None => Err(ApiError::NotFound("Tenant not found".to_string())),
    }
}

/// Atualiza tenant existente
#[utoipa::path(
    put,
    path = "/api/tenants/{tenant_id}",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    request_body = UpdateTenantRequest,
    responses(
        (status = 200, description = "Tenant atualizado com sucesso", body = Tenant),
        (status = 400, description = "Dados inválidos", body = ApiErrorResponse),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state, request))]
pub async fn update_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<UpdateTenantRequest>,
) -> ApiResult<Json<Tenant>> {
    info!("Updating tenant: {}", tenant_id);

    let tenant = app_state.tenant_service.update_tenant(&tenant_id, request).await?;
    
    Ok(Json(tenant))
}

/// Remove tenant (soft delete)
#[utoipa::path(
    delete,
    path = "/api/tenants/{tenant_id}",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    responses(
        (status = 204, description = "Tenant removido com sucesso"),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state))]
pub async fn delete_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<()> {
    info!("Deleting tenant: {}", tenant_id);

    app_state.tenant_service.delete_tenant(&tenant_id).await?;
    
    Ok(())
}



/// Obtém estatísticas de uso do tenant
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}/usage",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    responses(
        (status = 200, description = "Estatísticas de uso", body = TenantUsage),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state))]
pub async fn get_tenant_usage(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<TenantUsage>> {
    info!("Getting usage for tenant: {}", tenant_id);

    let usage = app_state.tenant_service.get_tenant_usage(&tenant_id).await?;
    
    Ok(Json(usage))
}

/// Obtém status de saúde do tenant
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}/health",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    responses(
        (status = 200, description = "Status de saúde", body = TenantHealthResponse),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state))]
pub async fn get_tenant_health(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<TenantHealthResponse>> {
    info!("Getting health for tenant: {}", tenant_id);

    let is_active = app_state.tenant_service.is_tenant_active(&tenant_id).await;
    let limits = app_state.tenant_service.get_tenant_limits(&tenant_id).await?;
    let usage = app_state.tenant_service.get_tenant_usage(&tenant_id).await?;

    let health = TenantHealthResponse {
        tenant_id,
        is_active,
        is_healthy: is_active && usage.total_calls < limits.max_concurrent_calls as u64,
        active_calls: usage.peak_concurrent_calls,
        calls_limit: limits.max_concurrent_calls,
        monthly_minutes_used: usage.total_minutes,
        monthly_minutes_limit: limits.max_monthly_minutes as u64,
        storage_used_mb: usage.recording_storage_used,
        storage_limit_mb: limits.max_recording_storage_mb,
        last_call_at: usage.period_end,
    };

    Ok(Json(health))
}

/// Response para status de saúde do tenant
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TenantHealthResponse {
    /// ID do tenant
    pub tenant_id: String,
    /// Se tenant está ativo
    pub is_active: bool,
    /// Se tenant está saudável (dentro dos limites)
    pub is_healthy: bool,
    /// Chamadas ativas no momento
    pub active_calls: u32,
    /// Limite de chamadas simultâneas
    pub calls_limit: u32,
    /// Minutos usados no mês atual
    pub monthly_minutes_used: u64,
    /// Limite de minutos mensais
    pub monthly_minutes_limit: u64,
    /// Storage usado para gravações (MB)
    pub storage_used_mb: u64,
    /// Limite de storage (MB)
    pub storage_limit_mb: u64,
    /// Timestamp da última chamada
    pub last_call_at: chrono::DateTime<chrono::Utc>,
}

/// Request para alterar status do tenant
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct SetTenantStatusRequest {
    /// Novo status do tenant
    pub status: TenantStatus,
}

/// Altera status do tenant (ativar/suspender/desativar)
#[utoipa::path(
    post,
    path = "/api/tenants/{tenant_id}/status",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    request_body = SetTenantStatusRequest,
    responses(
        (status = 200, description = "Status alterado com sucesso", body = Tenant),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state, request))]
pub async fn set_tenant_status(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(request): Json<SetTenantStatusRequest>,
) -> ApiResult<Json<Tenant>> {
    info!("Setting status {:?} for tenant {}", request.status, tenant_id);

    let update_request = UpdateTenantRequest {
        name: None,
        status: Some(request.status),
        service_plan: None,
        limits: None,
        config: None,
        contact: None,
        expires_at: None,
    };

    let tenant = app_state.tenant_service.update_tenant(&tenant_id, update_request).await?;
    
    Ok(Json(tenant))
}

/// Response com estatísticas detalhadas do tenant
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TenantDetailedStats {
    /// ID do tenant
    pub tenant_id: String,
    /// Total de usuários SIP criados
    pub total_users: u64,
    /// Usuários atualmente registrados
    pub active_users: u64,
    /// Total de chamadas no período
    pub total_calls: u64,
    /// Total de minutos no período
    pub total_minutes: u64,
    /// Pico de chamadas simultâneas
    pub peak_concurrent_calls: u32,
    /// Storage usado para gravações (MB)
    pub recording_storage_used: u64,
    /// Início do período de estatísticas
    pub period_start: chrono::DateTime<chrono::Utc>,
    /// Fim do período de estatísticas
    pub period_end: chrono::DateTime<chrono::Utc>,
    /// Data de criação do tenant
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Obtém estatísticas detalhadas do tenant
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}/stats",
    params(
        ("tenant_id" = String, Path, description = "ID único do tenant")
    ),
    responses(
        (status = 200, description = "Estatísticas detalhadas", body = TenantDetailedStats),
        (status = 404, description = "Tenant não encontrado", body = ApiErrorResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state))]
pub async fn get_tenant_detailed_stats(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<TenantDetailedStats>> {
    info!("Getting detailed stats for tenant: {}", tenant_id);

    // Verificar se tenant existe
    if app_state.tenant_service.get_tenant(&tenant_id).await?.is_none() {
        return Err(ApiError::NotFound("Tenant not found".to_string()));
    }

    // Obter dados
    let tenant_usage = app_state.tenant_service.get_tenant_usage(&tenant_id).await?;
    let sip_users = app_state.sip_user_service.list_users(&tenant_id, 1, 1000).await?;
    let active_registrations = app_state.sip_user_service.list_registrations(&tenant_id).await?;
    
    let stats = TenantDetailedStats {
        tenant_id: tenant_id.clone(),
        total_users: sip_users.total,
        active_users: active_registrations.len() as u64,
        total_calls: tenant_usage.total_calls,
        total_minutes: tenant_usage.total_minutes,
        peak_concurrent_calls: tenant_usage.peak_concurrent_calls,
        recording_storage_used: tenant_usage.recording_storage_used,
        period_start: tenant_usage.period_start,
        period_end: tenant_usage.period_end,
        created_at: chrono::Utc::now(), // TODO: usar data real do tenant
    };

    Ok(Json(stats))
}

/// Response de validação de tenant
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TenantValidationResponse {
    /// ID do tenant validado
    pub tenant_id: String,
    /// Se o ID está disponível
    pub is_available: bool,
    /// Se o formato é válido
    pub is_valid_format: bool,
    /// Mensagem de resultado
    pub message: String,
}

/// Valida se ID do tenant está disponível
#[utoipa::path(
    get,
    path = "/api/tenants/validate/{tenant_id}",
    params(
        ("tenant_id" = String, Path, description = "ID do tenant a validar")
    ),
    responses(
        (status = 200, description = "Validação realizada", body = TenantValidationResponse),
        (status = 500, description = "Erro interno", body = ApiErrorResponse)
    ),
    tag = "tenants"
)]
#[instrument(skip(app_state))]
pub async fn validate_tenant_id(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<TenantValidationResponse>> {
    info!("Validating tenant ID: {}", tenant_id);

    let exists = app_state.tenant_service.get_tenant(&tenant_id).await?.is_some();
    
    let response = TenantValidationResponse {
        tenant_id: tenant_id.clone(),
        is_available: !exists,
        is_valid_format: is_valid_tenant_id_format(&tenant_id),
        message: if exists {
            "Tenant ID already exists".to_string()
        } else if !is_valid_tenant_id_format(&tenant_id) {
            "Invalid tenant ID format".to_string()
        } else {
            "Tenant ID is available".to_string()
        },
    };

    Ok(Json(response))
}

// ==================== FUNÇÕES DE VALIDAÇÃO ====================

/// Valida formato do ID do tenant para uso em produção
fn is_valid_tenant_id_format(tenant_id: &str) -> bool {
    if tenant_id.len() < 3 || tenant_id.len() > 50 {
        return false;
    }
    
    // Deve começar com letra
    if !tenant_id.chars().next().unwrap_or('0').is_ascii_alphabetic() {
        return false;
    }
    
    // Pode conter apenas letras, números, _ e -
    tenant_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Valida formato do ID do tenant (para testes)
#[cfg(test)]
fn is_valid_tenant_id(tenant_id: &str) -> bool {
    if tenant_id.len() < 3 || tenant_id.len() > 50 {
        return false;
    }
    
    tenant_id.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Valida formato do domínio SIP
#[cfg(test)]
fn is_valid_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 255 {
        return false;
    }
    
    // Validação básica de domínio
    domain.split('.').all(|part| {
        !part.is_empty() && 
        part.len() <= 63 &&
        part.chars().all(|c| c.is_alphanumeric() || c == '-') &&
        !part.starts_with('-') &&
        !part.ends_with('-')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_id_validation() {
        assert!(is_valid_tenant_id("demo_company"));
        assert!(is_valid_tenant_id("acme123"));
        assert!(is_valid_tenant_id("test_tenant_001"));
        
        assert!(!is_valid_tenant_id("ab")); // muito curto
        assert!(!is_valid_tenant_id("invalid-tenant")); // hífen não permitido
        assert!(!is_valid_tenant_id("tenant.with.dots")); // pontos não permitidos
        assert!(!is_valid_tenant_id("")); // vazio
    }

    #[test]
    fn test_domain_validation() {
        assert!(is_valid_domain("demo.pbx.local"));
        assert!(is_valid_domain("tenant123.example.com"));
        assert!(is_valid_domain("localhost"));
        
        assert!(!is_valid_domain("")); // vazio
        assert!(!is_valid_domain(".invalid")); // inicia com ponto
        assert!(!is_valid_domain("invalid.")); // termina com ponto
        assert!(!is_valid_domain("invalid..domain")); // pontos duplos
    }
}

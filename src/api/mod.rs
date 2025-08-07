//! 🌐 API Layer - REST API com Swagger

pub mod handlers;
pub mod middleware;

use crate::config::AppConfig;
use crate::services::{CallManager, SipUserService, TenantService};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

/// Documentação OpenAPI
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health_check,
        handlers::calls::list_calls,
        handlers::calls::get_call,
        handlers::calls::originate_call,
        handlers::calls::hangup_call,
        handlers::status::get_status,
        handlers::status::get_statistics,
        handlers::tenants::list_tenants,
        handlers::tenants::create_tenant,
        handlers::tenants::get_tenant,
        handlers::tenants::update_tenant,
        handlers::tenants::delete_tenant,
        handlers::tenants::get_tenant_usage,
        handlers::tenants::get_tenant_health,
        handlers::tenants::set_tenant_status,
        handlers::tenants::get_tenant_detailed_stats,
        handlers::tenants::validate_tenant_id,
        handlers::sip_users::list_sip_users,
        handlers::sip_users::create_sip_user,
        handlers::sip_users::get_sip_user,
        handlers::sip_users::get_sip_user_by_extension,
        handlers::sip_users::update_sip_user,
        handlers::sip_users::delete_sip_user,
        handlers::sip_users::reset_sip_user_password,
        handlers::sip_users::set_sip_user_status,
        handlers::sip_users::list_sip_registrations,
        handlers::sip_users::get_sip_user_usage,
        handlers::sip_users::authenticate_sip_user,
    ),
    components(
        schemas(
            crate::models::Call,
            crate::models::CallStatus,
            crate::models::CallListResponse,
            crate::models::CallSummary,
            crate::models::OriginateCallRequest,
            crate::models::ErrorResponse,
            crate::models::ApiErrorResponse,
            crate::api::handlers::health::HealthResponse,
            crate::api::handlers::status::SystemStatusResponse,
            crate::api::handlers::status::StatisticsResponse,
            crate::api::handlers::status::ApplicationInfo,
            crate::api::handlers::status::ServerInfo,
            crate::models::Tenant,
            crate::models::CreateTenantRequest,
            crate::models::UpdateTenantRequest,
            crate::models::TenantConfig,
            crate::models::TenantConfigRequest,
            crate::models::BillingSettings,
            crate::models::ConferenceSettings,
            crate::models::IvrSettings,
            crate::models::RecordingPolicy,
            crate::models::TenantContact,
            crate::models::TenantLimits,
            crate::models::ServicePlan,
            crate::models::TenantStatus,
            crate::models::TenantSummary,
            crate::models::UsageCosts,
            crate::models::TenantListResponse,
            crate::models::TenantUsage,
            crate::api::handlers::tenants::TenantHealthResponse,
            crate::api::handlers::tenants::SetTenantStatusRequest,
            crate::api::handlers::tenants::TenantDetailedStats,
            crate::api::handlers::tenants::TenantValidationResponse,
            crate::models::SipUser,
            crate::models::SipUserStatus,
            crate::models::SipUserConfig,
            crate::models::SipUserLimits,
            crate::models::SipUserResponse,
            crate::models::SipUserListResponse,
            crate::models::CreateSipUserRequest,
            crate::models::UpdateSipUserRequest,
            crate::models::SipRegistrationInfo,
            crate::models::SipUserUsageStats,
            crate::api::handlers::sip_users::ResetPasswordRequest,
            crate::api::handlers::sip_users::SetUserStatusRequest,
            crate::api::handlers::sip_users::SipAuthRequest,
            crate::api::handlers::sip_users::SipAuthResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "calls", description = "Call management endpoints"),
        (name = "status", description = "System status endpoints"),
        (name = "tenants", description = "Multi-tenant management endpoints"),
        (name = "sip-users", description = "SIP user management endpoints"),
        (name = "sip-auth", description = "SIP authentication endpoints"),
    ),
    info(
        title = "Backend PBX API",
        version = "0.2.0",
        description = "API REST para controle de chamadas VoIP via FreeSWITCH",
        contact(
            name = "Backend PBX",
            email = "contato@backendpbx.com"
        )
    )
)]
pub struct ApiDoc;

/// Addon para adicionar schemas de tipos externos
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::{RefOr, Schema, SchemaType, SchemaFormat};
        
        if let Some(components) = openapi.components.as_mut() {
            // Adicionar schema para DateTime
            let datetime_schema = Schema::Object(
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .format(Some(SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::DateTime)))
                    .description(Some("ISO 8601 date-time format"))
                    .example(Some(serde_json::json!("2025-01-07T01:30:25.189834600Z")))
                    .build()
            );
            
            components.schemas.insert(
                "DateTime".to_string(),
                RefOr::T(datetime_schema),
            );
            
            // Adicionar schema para Uuid
            let uuid_schema = Schema::Object(
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .format(Some(SchemaFormat::Custom("uuid".to_string())))
                    .description(Some("UUID v4 format"))
                    .example(Some(serde_json::json!("550e8400-e29b-41d4-a716-446655440000")))
                    .build()
            );
            
            components.schemas.insert(
                "Uuid".to_string(),
                RefOr::T(uuid_schema),
            );
            
            // Adicionar schema para chrono.DateTime (alias para DateTime)
            let chrono_datetime_schema = Schema::Object(
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .format(Some(SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::DateTime)))
                    .description(Some("ISO 8601 date-time format (chrono)"))
                    .example(Some(serde_json::json!("2025-01-07T01:30:25.189834600Z")))
                    .build()
            );
            
            components.schemas.insert(
                "chrono.DateTime".to_string(),
                RefOr::T(chrono_datetime_schema),
            );
        }
    }
}

/// Estado compartilhado da aplicação
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub call_manager: Arc<CallManager>,
    pub tenant_service: Arc<dyn TenantService>,
    pub sip_user_service: Arc<dyn SipUserService>,
}

/// Cria o router principal da API
pub fn create_router(state: AppState) -> Router {
    // Configurar CORS
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    // Middleware stack
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        
        // API v1 routes
        .nest("/api/v1", api_v1_routes())
        
        // Tenant management routes
        .nest("/api", tenant_routes())
        
        // SIP user routes
        .nest("/api", sip_user_routes())
        
        // SIP authentication routes (for FreeSWITCH)
        .nest("/api", sip_auth_routes())
        
        // FreeSWITCH integration routes
        .nest("/api", freeswitch_routes())
        
        // 📚 Code-First OpenAPI - Documentação gerada automaticamente do código!
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        
        // 📊 Informações sobre a geração automática da documentação
        .route("/api-docs/info", get(serve_openapi_info))
        
        // Apply middleware
        .layer(middleware)
        
        // Add shared state
        .with_state(state)
}

/// Routes da API v1
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Call management
        .route("/calls", get(handlers::calls::list_calls))
        .route("/calls", post(handlers::calls::originate_call))
        .route("/calls/:id", get(handlers::calls::get_call))
        .route("/calls/:id/hangup", post(handlers::calls::hangup_call))
        
        // System status
        .route("/status", get(handlers::status::get_status))
        .route("/statistics", get(handlers::status::get_statistics))
}

/// Routes de gerenciamento de tenants
fn tenant_routes() -> Router<AppState> {
    Router::new()
        // CRUD de tenants
        .route("/tenants", get(handlers::tenants::list_tenants))
        .route("/tenants", post(handlers::tenants::create_tenant))
        .route("/tenants/:tenant_id", get(handlers::tenants::get_tenant))
        .route("/tenants/:tenant_id", put(handlers::tenants::update_tenant))
        .route("/tenants/:tenant_id", delete(handlers::tenants::delete_tenant))
        
        // Operações especiais de tenants
        .route("/tenants/:tenant_id/usage", get(handlers::tenants::get_tenant_usage))
        .route("/tenants/:tenant_id/health", get(handlers::tenants::get_tenant_health))
        .route("/tenants/:tenant_id/status", post(handlers::tenants::set_tenant_status))
        .route("/tenants/:tenant_id/stats", get(handlers::tenants::get_tenant_detailed_stats))
        
        // Validação de tenants
        .route("/tenants/validate/:tenant_id", get(handlers::tenants::validate_tenant_id))
}

/// Routes de gerenciamento de usuários SIP
fn sip_user_routes() -> Router<AppState> {
    Router::new()
        // CRUD de usuários SIP
        .route("/tenants/:tenant_id/sip-users", get(handlers::sip_users::list_sip_users))
        .route("/tenants/:tenant_id/sip-users", post(handlers::sip_users::create_sip_user))
        .route("/tenants/:tenant_id/sip-users/:user_id", get(handlers::sip_users::get_sip_user))
        .route("/tenants/:tenant_id/sip-users/:user_id", put(handlers::sip_users::update_sip_user))
        .route("/tenants/:tenant_id/sip-users/:user_id", delete(handlers::sip_users::delete_sip_user))
        
        // Busca por extensão
        .route("/tenants/:tenant_id/sip-users/extension/:extension", get(handlers::sip_users::get_sip_user_by_extension))
        
        // Operações especiais
        .route("/tenants/:tenant_id/sip-users/:user_id/reset-password", post(handlers::sip_users::reset_sip_user_password))
        .route("/tenants/:tenant_id/sip-users/:user_id/status", post(handlers::sip_users::set_sip_user_status))
        .route("/tenants/:tenant_id/sip-users/:user_id/usage", get(handlers::sip_users::get_sip_user_usage))
        
        // Registros SIP
        .route("/tenants/:tenant_id/sip-users/registrations", get(handlers::sip_users::list_sip_registrations))
}

/// Routes de autenticação SIP (para FreeSWITCH)
fn sip_auth_routes() -> Router<AppState> {
    Router::new()
        .route("/sip/auth/:tenant_id/:extension", post(handlers::sip_users::authenticate_sip_user))
}

/// Routes de integração com FreeSWITCH
fn freeswitch_routes() -> Router<AppState> {
    Router::new()
        .route("/freeswitch/directory", get(handlers::freeswitch::freeswitch_directory))
        .route("/freeswitch/directory", post(handlers::freeswitch::freeswitch_directory))
        .route("/freeswitch/auth", get(handlers::freeswitch::freeswitch_auth))
        .route("/freeswitch/auth", post(handlers::freeswitch::freeswitch_auth))
}



/// 📊 Informações sobre a geração automática da documentação
async fn serve_openapi_info() -> axum::response::Json<serde_json::Value> {
    use chrono::Utc;
    
    let openapi = ApiDoc::openapi();
    let generated_at = Utc::now();
    
    axum::response::Json(serde_json::json!({
        "message": "🚀 Code-First OpenAPI Documentation",
        "description": "Esta documentação é gerada automaticamente a partir das anotações no código Rust",
        "approach": "Code-First",
        "generated_at": generated_at,
        "statistics": {
            "total_paths": openapi.paths.paths.len(),
            "total_endpoints": openapi.paths.paths.values()
                .map(|p| p.operations.len())
                .sum::<usize>(),
            "total_schemas": openapi.components.as_ref()
                .map(|c| c.schemas.len())
                .unwrap_or(0),
            "total_tags": openapi.tags.as_ref()
                .map(|t| t.len())
                .unwrap_or(0),
        },
        "benefits": [
            "✅ Sincronização automática entre código e documentação",
            "✅ Zero manutenção manual necessária", 
            "✅ Documentação sempre atualizada",
            "✅ Type safety garantido",
            "✅ Impossível ter divergências"
        ],
        "endpoints": {
            "swagger_ui": "/swagger-ui",
            "openapi_json": "/api-docs/openapi.json",
            "info": "/api-docs/info"
        },
        "version": openapi.info.version,
        "title": openapi.info.title
    }))
}

//! 🏢 Tenant Service - Gerenciamento de tenants multi-tenant
//! 
//! Este serviço é responsável por gerenciar todas as operações relacionadas
//! a tenants, incluindo validação, limites, configurações e métricas.

use crate::models::{
    ApiError, ApiResult, Tenant, TenantUsage, TenantStatus,
    CreateTenantRequest, UpdateTenantRequest, TenantListResponse
};
use crate::services::esl::{TenantLimits as EslTenantLimits};

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

/// Trait para operações de gerenciamento de tenants
#[async_trait]
pub trait TenantService: Send + Sync {
    /// Cria novo tenant
    async fn create_tenant(&self, request: CreateTenantRequest) -> ApiResult<Tenant>;
    
    /// Obtém tenant por ID
    async fn get_tenant(&self, tenant_id: &str) -> ApiResult<Option<Tenant>>;
    
    /// Lista tenants com paginação
    async fn list_tenants(&self, page: u32, per_page: u32) -> ApiResult<TenantListResponse>;
    
    /// Atualiza tenant
    async fn update_tenant(&self, tenant_id: &str, request: UpdateTenantRequest) -> ApiResult<Tenant>;
    
    /// Remove tenant (soft delete)
    async fn delete_tenant(&self, tenant_id: &str) -> ApiResult<()>;
    
    /// Valida se tenant pode executar operação
    async fn validate_tenant_operation(&self, tenant_id: &str, operation: &str) -> ApiResult<()>;
    
    /// Obtém limites do tenant
    async fn get_tenant_limits(&self, tenant_id: &str) -> ApiResult<EslTenantLimits>;
    
    /// Obtém uso atual do tenant
    async fn get_tenant_usage(&self, tenant_id: &str) -> ApiResult<TenantUsage>;
    
    /// Verifica se tenant existe e está ativo
    async fn is_tenant_active(&self, tenant_id: &str) -> bool;
    
    /// Obtém tenant por domínio SIP
    async fn get_tenant_by_domain(&self, domain: &str) -> ApiResult<Option<String>>;
}

/// Implementação em memória do serviço de tenants
/// Em produção, seria substituída por implementação com banco de dados
pub struct TenantServiceImpl {
    /// Storage de tenants em memória
    tenants: Arc<RwLock<HashMap<String, Tenant>>>,
    /// Índice por domínio SIP
    domain_index: Arc<RwLock<HashMap<String, String>>>,
    /// Cache de uso de recursos
    usage_cache: Arc<RwLock<HashMap<String, TenantUsage>>>,
    /// Metrics de chamadas ativas por tenant
    active_calls: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    /// Rate limiting - timestamp da última chamada por tenant
    rate_limits: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl TenantServiceImpl {
    /// Cria nova instância do serviço
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
            usage_cache: Arc::new(RwLock::new(HashMap::new())),
            active_calls: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Inicializa com tenants padrão para demonstração
    pub async fn initialize_demo_tenants(&self) -> ApiResult<()> {
        // Tenant de demonstração 1
        let demo_tenant_1 = CreateTenantRequest {
            id: "demo_company".to_string(),
            name: "Demo Company LTDA".to_string(),
            service_plan: crate::models::ServicePlan::Professional,
            sip_domain: "demo.pbx.local".to_string(),
            contact: crate::models::TenantContact {
                company_name: "Demo Company LTDA".to_string(),
                contact_name: "João Silva".to_string(),
                email: "joao@democompany.com".to_string(),
                phone: "+5511999999999".to_string(),
                address: Some("Rua Demo, 123".to_string()),
                city: Some("São Paulo".to_string()),
                state: Some("SP".to_string()),
                postal_code: Some("01234-567".to_string()),
                country: "Brasil".to_string(),
            },
            config: None,
            custom_limits: None,
        };

        // Tenant de demonstração 2
        let demo_tenant_2 = CreateTenantRequest {
            id: "acme_corp".to_string(),
            name: "ACME Corporation".to_string(),
            service_plan: crate::models::ServicePlan::Enterprise,
            sip_domain: "acme.pbx.local".to_string(),
            contact: crate::models::TenantContact {
                company_name: "ACME Corporation".to_string(),
                contact_name: "Maria Santos".to_string(),
                email: "maria@acme.com".to_string(),
                phone: "+5511888888888".to_string(),
                address: Some("Av. Acme, 456".to_string()),
                city: Some("Rio de Janeiro".to_string()),
                state: Some("RJ".to_string()),
                postal_code: Some("20000-000".to_string()),
                country: "Brasil".to_string(),
            },
            config: Some(crate::models::TenantConfigRequest {
                default_gateway: Some("provider_1".to_string()),
                recording_policy: Some(crate::models::RecordingPolicy::Always),
                presence_enabled: Some(true),
                conference_settings: None,
                ivr_settings: None,
                timezone: Some("America/Sao_Paulo".to_string()),
                billing_settings: None,
            }),
            custom_limits: None,
        };

        self.create_tenant(demo_tenant_1).await?;
        self.create_tenant(demo_tenant_2).await?;

        info!("Demo tenants initialized successfully");
        Ok(())
    }

    /// Atualiza métricas de chamadas ativas
    pub async fn add_active_call(&self, tenant_id: &str, call_uuid: Uuid) {
        let mut active_calls = self.active_calls.write().await;
        active_calls.entry(tenant_id.to_string())
            .or_insert_with(Vec::new)
            .push(call_uuid);
    }

    /// Remove chamada ativa
    pub async fn remove_active_call(&self, tenant_id: &str, call_uuid: Uuid) {
        let mut active_calls = self.active_calls.write().await;
        if let Some(calls) = active_calls.get_mut(tenant_id) {
            calls.retain(|&id| id != call_uuid);
        }
    }

    /// Obtém número de chamadas ativas do tenant
    pub async fn get_active_calls_count(&self, tenant_id: &str) -> u32 {
        let active_calls = self.active_calls.read().await;
        active_calls.get(tenant_id)
            .map(|calls| calls.len() as u32)
            .unwrap_or(0)
    }

    /// Verifica rate limiting para tenant
    async fn check_rate_limit(&self, tenant_id: &str) -> ApiResult<()> {
        let tenant = self.get_tenant(tenant_id).await?
            .ok_or_else(|| ApiError::BadRequest("Tenant not found".to_string()))?;

        let now = Utc::now();
        let minute_ago = now - Duration::minutes(1);

        let mut rate_limits = self.rate_limits.write().await;
        let calls_in_last_minute = rate_limits.entry(tenant_id.to_string())
            .or_insert_with(Vec::new);

        // Remove entradas antigas
        calls_in_last_minute.retain(|&timestamp| timestamp > minute_ago);

        // Verificar limite
        if calls_in_last_minute.len() >= tenant.limits.max_calls_per_minute as usize {
            return Err(ApiError::TooManyRequests(
                "Rate limit exceeded for tenant".to_string()
            ));
        }

        // Adicionar chamada atual
        calls_in_last_minute.push(now);

        Ok(())
    }

    /// Gera uso simulado para demonstração
    async fn generate_demo_usage(&self, tenant_id: &str) -> TenantUsage {
        let now = Utc::now();
        let period_start = now - Duration::days(30);
        
        // Simular dados baseados no tenant
        let (total_calls, total_minutes) = match tenant_id {
            "demo_company" => (245, 1850),
            "acme_corp" => (1420, 8500),
            _ => (100, 500),
        };

        TenantUsage {
            tenant_id: tenant_id.to_string(),
            period_start,
            period_end: now,
            total_calls,
            total_minutes,
            peak_concurrent_calls: self.get_active_calls_count(tenant_id).await,
            answer_seizure_ratio: 0.85,
            average_call_duration: (total_minutes * 60) as f64 / total_calls as f64,
            recording_storage_used: total_minutes / 10, // ~10% das chamadas gravadas
            costs: crate::models::UsageCosts {
                local_calls: (total_minutes as f64 * 0.05),
                national_calls: (total_minutes as f64 * 0.15),
                international_calls: (total_minutes as f64 * 0.50),
                storage: (total_minutes as f64 / 10.0 * 0.01),
                total: (total_minutes as f64 * 0.15),
                currency: "BRL".to_string(),
            },
        }
    }
}

#[async_trait]
impl TenantService for TenantServiceImpl {
    #[instrument(skip(self, request))]
    async fn create_tenant(&self, request: CreateTenantRequest) -> ApiResult<Tenant> {
        // Validar que ID é único
        {
            let tenants = self.tenants.read().await;
            if tenants.contains_key(&request.id) {
                return Err(ApiError::BadRequest("Tenant ID already exists".to_string()));
            }
        }

        // Validar que domínio é único
        {
            let domain_index = self.domain_index.read().await;
            if domain_index.contains_key(&request.sip_domain) {
                return Err(ApiError::BadRequest("SIP domain already in use".to_string()));
            }
        }

        // Criar tenant
        let tenant = Tenant::new(request.clone());
        let tenant_id = tenant.id.clone();
        let sip_domain = tenant.config.sip_domain.clone();

        // Armazenar tenant
        {
            let mut tenants = self.tenants.write().await;
            tenants.insert(tenant_id.clone(), tenant.clone());
        }

        // Atualizar índice de domínio
        {
            let mut domain_index = self.domain_index.write().await;
            domain_index.insert(sip_domain, tenant_id.clone());
        }

        info!("Created tenant: {} ({})", tenant.name, tenant_id);
        Ok(tenant)
    }

    #[instrument(skip(self))]
    async fn get_tenant(&self, tenant_id: &str) -> ApiResult<Option<Tenant>> {
        let tenants = self.tenants.read().await;
        Ok(tenants.get(tenant_id).cloned())
    }

    #[instrument(skip(self))]
    async fn list_tenants(&self, page: u32, per_page: u32) -> ApiResult<TenantListResponse> {
        let tenants = self.tenants.read().await;
        let total = tenants.len();
        
        let start = ((page - 1) * per_page) as usize;
        let end = (start + per_page as usize).min(total);
        
        let mut tenant_summaries = Vec::new();
        for (_, tenant) in tenants.iter().skip(start).take(end - start) {
            let active_calls = self.get_active_calls_count(&tenant.id).await;
            // TODO: Em produção, buscar número real de extensões ativas
            let active_extensions = match tenant.service_plan {
                crate::models::ServicePlan::Basic => 5,
                crate::models::ServicePlan::Professional => 25,
                crate::models::ServicePlan::Enterprise => 80,
                crate::models::ServicePlan::Custom => 10,
            };
            
            tenant_summaries.push(tenant.to_summary(active_extensions, active_calls));
        }

        Ok(TenantListResponse {
            tenants: tenant_summaries,
            total,
            page,
            per_page,
        })
    }

    #[instrument(skip(self, request))]
    async fn update_tenant(&self, tenant_id: &str, request: UpdateTenantRequest) -> ApiResult<Tenant> {
        let mut tenants = self.tenants.write().await;
        
        match tenants.get_mut(tenant_id) {
            Some(tenant) => {
                tenant.update(request);
                info!("Updated tenant: {}", tenant_id);
                Ok(tenant.clone())
            }
            None => Err(ApiError::NotFound("Tenant not found".to_string()))
        }
    }

    #[instrument(skip(self))]
    async fn delete_tenant(&self, tenant_id: &str) -> ApiResult<()> {
        // Em produção, seria soft delete
        let mut tenants = self.tenants.write().await;
        
        match tenants.get_mut(tenant_id) {
            Some(tenant) => {
                tenant.status = TenantStatus::Inactive;
                info!("Deactivated tenant: {}", tenant_id);
                Ok(())
            }
            None => Err(ApiError::NotFound("Tenant not found".to_string()))
        }
    }

    #[instrument(skip(self))]
    async fn validate_tenant_operation(&self, tenant_id: &str, operation: &str) -> ApiResult<()> {
        let tenant = self.get_tenant(tenant_id).await?
            .ok_or_else(|| ApiError::BadRequest("Tenant not found".to_string()))?;

        // Verificar se tenant está ativo
        if !tenant.is_active() {
            return Err(ApiError::Forbidden("Tenant is not active".to_string()));
        }

        // Verificar se tenant expirou
        if tenant.is_expired() {
            return Err(ApiError::Forbidden("Tenant has expired".to_string()));
        }

        // Verificar rate limiting para operações de chamada
        if operation == "originate_call" {
            self.check_rate_limit(tenant_id).await?;
        }

        // Verificar limites de chamadas simultâneas
        if operation == "originate_call" {
            let active_calls = self.get_active_calls_count(tenant_id).await;
            if active_calls >= tenant.limits.max_concurrent_calls {
                return Err(ApiError::TooManyRequests(
                    "Concurrent call limit reached".to_string()
                ));
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_tenant_limits(&self, tenant_id: &str) -> ApiResult<EslTenantLimits> {
        let tenant = self.get_tenant(tenant_id).await?
            .ok_or_else(|| ApiError::BadRequest("Tenant not found".to_string()))?;

        Ok(EslTenantLimits {
            max_concurrent_calls: tenant.limits.max_concurrent_calls,
            max_extensions: tenant.limits.max_extensions,
            max_monthly_minutes: tenant.limits.max_monthly_minutes,
            max_recording_storage_mb: tenant.limits.max_recording_storage_mb,
        })
    }

    #[instrument(skip(self))]
    async fn get_tenant_usage(&self, tenant_id: &str) -> ApiResult<TenantUsage> {
        // Verificar se tenant existe
        self.get_tenant(tenant_id).await?
            .ok_or_else(|| ApiError::BadRequest("Tenant not found".to_string()))?;

        // Em produção, buscaria do banco de dados
        // Por enquanto, gerar dados de demonstração
        let usage = self.generate_demo_usage(tenant_id).await;
        
        // Cache do resultado
        {
            let mut usage_cache = self.usage_cache.write().await;
            usage_cache.insert(tenant_id.to_string(), usage.clone());
        }

        Ok(usage)
    }

    #[instrument(skip(self))]
    async fn is_tenant_active(&self, tenant_id: &str) -> bool {
        if let Ok(Some(tenant)) = self.get_tenant(tenant_id).await {
            tenant.is_active() && !tenant.is_expired()
        } else {
            false
        }
    }

    #[instrument(skip(self))]
    async fn get_tenant_by_domain(&self, domain: &str) -> ApiResult<Option<String>> {
        let domain_index = self.domain_index.read().await;
        Ok(domain_index.get(domain).cloned())
    }
}

/// Middleware para validação de tenant em requisições
pub struct TenantMiddleware {
    tenant_service: Arc<dyn TenantService>,
}

impl TenantMiddleware {
    pub fn new(tenant_service: Arc<dyn TenantService>) -> Self {
        Self { tenant_service }
    }

    /// Extrai tenant_id do header ou path da requisição
    pub fn extract_tenant_id(_request: &axum::http::Request<axum::body::Body>) -> Option<String> {
        // TODO: Implementar extração real do tenant_id
        // Pode vir de header X-Tenant-ID, subdomain, path parameter, etc.
        Some("demo_company".to_string())
    }

    /// Valida se tenant pode executar operação
    pub async fn validate_tenant_request(
        &self,
        tenant_id: &str,
        operation: &str,
    ) -> ApiResult<()> {
        self.tenant_service.validate_tenant_operation(tenant_id, operation).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ServicePlan, TenantContact};

    #[tokio::test]
    async fn test_tenant_service() {
        let service = TenantServiceImpl::new();

        let request = CreateTenantRequest {
            id: "test_tenant".to_string(),
            name: "Test Company".to_string(),
            service_plan: ServicePlan::Professional,
            sip_domain: "test.local".to_string(),
            contact: TenantContact {
                company_name: "Test Company".to_string(),
                contact_name: "Test User".to_string(),
                email: "test@test.com".to_string(),
                phone: "+5511999999999".to_string(),
                address: None,
                city: None,
                state: None,
                postal_code: None,
                country: "Brasil".to_string(),
            },
            config: None,
            custom_limits: None,
        };

        // Criar tenant
        let tenant = service.create_tenant(request).await.unwrap();
        assert_eq!(tenant.id, "test_tenant");
        assert!(tenant.is_active());

        // Buscar tenant
        let found = service.get_tenant("test_tenant").await.unwrap();
        assert!(found.is_some());

        // Verificar se está ativo
        assert!(service.is_tenant_active("test_tenant").await);

        // Validar operação
        assert!(service.validate_tenant_operation("test_tenant", "originate_call").await.is_ok());
    }
}

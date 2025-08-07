//! 🏢 Tenant Model - Estruturas para suporte multi-tenant
//! 
//! Este módulo define as estruturas de dados para gerenciar tenants
//! no sistema PABX, incluindo limites, configurações e validações.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use std::collections::HashMap;

/// Status de um tenant no sistema
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantStatus {
    /// Tenant ativo e operacional
    Active,
    /// Suspenso temporariamente (não pode fazer/receber chamadas)
    Suspended,
    /// Inativo/cancelado (sem acesso)
    Inactive,
    /// Em período de teste/trial
    Trial,
    /// Aguardando ativação
    Pending,
}

/// Plano de serviço do tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServicePlan {
    /// Plano básico
    Basic,
    /// Plano profissional
    Professional,
    /// Plano empresarial
    Enterprise,
    /// Plano personalizado
    Custom,
}

/// Limites de recursos por tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantLimits {
    /// Máximo de chamadas simultâneas
    pub max_concurrent_calls: u32,
    /// Máximo de ramais/extensões
    pub max_extensions: u32,
    /// Minutos máximos por mês
    pub max_monthly_minutes: u32,
    /// Espaço máximo para gravações (MB)
    pub max_recording_storage_mb: u64,
    /// Máximo de conferências simultâneas
    pub max_conferences: u32,
    /// Máximo de filas de atendimento
    pub max_queues: u32,
    /// Taxa máxima de chamadas por minuto (rate limiting)
    pub max_calls_per_minute: u32,
}

impl Default for TenantLimits {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 10,
            max_extensions: 50,
            max_monthly_minutes: 1000,
            max_recording_storage_mb: 1024,
            max_conferences: 3,
            max_queues: 5,
            max_calls_per_minute: 60,
        }
    }
}

impl TenantLimits {
    /// Cria limites baseados no plano de serviço
    pub fn from_service_plan(plan: &ServicePlan) -> Self {
        match plan {
            ServicePlan::Basic => Self {
                max_concurrent_calls: 5,
                max_extensions: 20,
                max_monthly_minutes: 500,
                max_recording_storage_mb: 512,
                max_conferences: 1,
                max_queues: 2,
                max_calls_per_minute: 30,
            },
            ServicePlan::Professional => Self {
                max_concurrent_calls: 20,
                max_extensions: 100,
                max_monthly_minutes: 2000,
                max_recording_storage_mb: 2048,
                max_conferences: 5,
                max_queues: 10,
                max_calls_per_minute: 120,
            },
            ServicePlan::Enterprise => Self {
                max_concurrent_calls: 100,
                max_extensions: 500,
                max_monthly_minutes: 10000,
                max_recording_storage_mb: 10240,
                max_conferences: 20,
                max_queues: 50,
                max_calls_per_minute: 300,
            },
            ServicePlan::Custom => Self::default(),
        }
    }
}

/// Configurações específicas do tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantConfig {
    /// Domínio SIP principal
    pub sip_domain: String,
    /// Context padrão no dialplan
    pub default_context: String,
    /// Gateway padrão para chamadas externas
    pub default_gateway: Option<String>,
    /// Política de gravação padrão
    pub recording_policy: RecordingPolicy,
    /// Configurações de presença
    pub presence_enabled: bool,
    /// Configurações de conferência
    pub conference_settings: ConferenceSettings,
    /// Configurações de IVR
    pub ivr_settings: IvrSettings,
    /// Timezone do tenant
    pub timezone: String,
    /// Configurações de billing
    pub billing_settings: BillingSettings,
}

/// Política de gravação
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingPolicy {
    /// Nunca gravar
    Never,
    /// Gravar todas as chamadas
    Always,
    /// Gravar apenas chamadas externas
    ExternalOnly,
    /// Gravar apenas chamadas internas
    InternalOnly,
    /// Gravação manual/sob demanda
    OnDemand,
}

/// Configurações de conferência
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConferenceSettings {
    /// Prefixo para salas de conferência
    pub room_prefix: String,
    /// Tamanho máximo de sala
    pub max_participants: u32,
    /// Gravação automática de conferências
    pub auto_record: bool,
    /// Moderação obrigatória
    pub moderated: bool,
}

impl Default for ConferenceSettings {
    fn default() -> Self {
        Self {
            room_prefix: "conf".to_string(),
            max_participants: 25,
            auto_record: false,
            moderated: false,
        }
    }
}

/// Configurações de IVR
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IvrSettings {
    /// Timeout entre dígitos (segundos)
    pub digit_timeout: u32,
    /// Timeout total (segundos)
    pub total_timeout: u32,
    /// Número máximo de tentativas
    pub max_retries: u32,
    /// Diretório de áudios personalizados
    pub audio_directory: String,
}

impl Default for IvrSettings {
    fn default() -> Self {
        Self {
            digit_timeout: 5,
            total_timeout: 30,
            max_retries: 3,
            audio_directory: "default".to_string(),
        }
    }
}

/// Configurações de billing
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BillingSettings {
    /// Moeda para cobrança
    pub currency: String,
    /// Taxa por minuto para chamadas locais
    pub local_rate_per_minute: f64,
    /// Taxa por minuto para chamadas nacionais
    pub national_rate_per_minute: f64,
    /// Taxa por minuto para chamadas internacionais
    pub international_rate_per_minute: f64,
    /// Incremento de billing (segundos)
    pub billing_increment: u32,
    /// Período de carência antes do billing
    pub grace_period_seconds: u32,
}

impl Default for BillingSettings {
    fn default() -> Self {
        Self {
            currency: "BRL".to_string(),
            local_rate_per_minute: 0.05,
            national_rate_per_minute: 0.15,
            international_rate_per_minute: 0.50,
            billing_increment: 60,  // 1 minuto
            grace_period_seconds: 30,
        }
    }
}

/// Informações de contato do tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantContact {
    /// Nome da empresa/organização
    pub company_name: String,
    /// Nome do contato principal
    pub contact_name: String,
    /// Email principal
    pub email: String,
    /// Telefone principal
    pub phone: String,
    /// Endereço
    pub address: Option<String>,
    /// Cidade
    pub city: Option<String>,
    /// Estado
    pub state: Option<String>,
    /// CEP
    pub postal_code: Option<String>,
    /// País
    pub country: String,
}

/// Informações completas de um tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Tenant {
    /// ID único do tenant
    pub id: String,
    /// Nome de exibição
    pub name: String,
    /// Status atual
    pub status: TenantStatus,
    /// Plano de serviço
    pub service_plan: ServicePlan,
    /// Limites de recursos
    pub limits: TenantLimits,
    /// Configurações específicas
    pub config: TenantConfig,
    /// Informações de contato
    pub contact: TenantContact,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Data da última atualização
    pub updated_at: DateTime<Utc>,
    /// Data de expiração (para trials)
    pub expires_at: Option<DateTime<Utc>>,
    /// Metadados adicionais
    pub metadata: HashMap<String, String>,
}

/// Request para criar novo tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    /// ID do tenant (deve ser único)
    pub id: String,
    /// Nome de exibição
    pub name: String,
    /// Plano de serviço
    pub service_plan: ServicePlan,
    /// Domínio SIP
    pub sip_domain: String,
    /// Informações de contato
    pub contact: TenantContact,
    /// Configurações específicas (opcional)
    pub config: Option<TenantConfigRequest>,
    /// Limites customizados (opcional, senão usa do plano)
    pub custom_limits: Option<TenantLimits>,
}

/// Request para configuração de tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantConfigRequest {
    /// Gateway padrão
    pub default_gateway: Option<String>,
    /// Política de gravação
    pub recording_policy: Option<RecordingPolicy>,
    /// Habilitar presença
    pub presence_enabled: Option<bool>,
    /// Configurações de conferência
    pub conference_settings: Option<ConferenceSettings>,
    /// Configurações de IVR
    pub ivr_settings: Option<IvrSettings>,
    /// Timezone
    pub timezone: Option<String>,
    /// Configurações de billing
    pub billing_settings: Option<BillingSettings>,
}

/// Request para atualizar tenant
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTenantRequest {
    /// Nome de exibição
    pub name: Option<String>,
    /// Status
    pub status: Option<TenantStatus>,
    /// Plano de serviço
    pub service_plan: Option<ServicePlan>,
    /// Limites customizados
    pub limits: Option<TenantLimits>,
    /// Configurações
    pub config: Option<TenantConfigRequest>,
    /// Informações de contato
    pub contact: Option<TenantContact>,
    /// Data de expiração
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response de listagem de tenants
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TenantListResponse {
    /// Lista de tenants
    pub tenants: Vec<TenantSummary>,
    /// Total de tenants
    pub total: usize,
    /// Página atual
    pub page: u32,
    /// Itens por página
    pub per_page: u32,
}

/// Resumo de um tenant para listagem
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TenantSummary {
    /// ID do tenant
    pub id: String,
    /// Nome
    pub name: String,
    /// Status
    pub status: TenantStatus,
    /// Plano
    pub service_plan: ServicePlan,
    /// Domínio SIP
    pub sip_domain: String,
    /// Número de extensões ativas
    pub active_extensions: u32,
    /// Chamadas ativas no momento
    pub active_calls: u32,
    /// Data de criação
    pub created_at: DateTime<Utc>,
}

/// Estatísticas de uso do tenant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantUsage {
    /// ID do tenant
    pub tenant_id: String,
    /// Período das estatísticas
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    /// Chamadas realizadas no período
    pub total_calls: u64,
    /// Minutos consumidos
    pub total_minutes: u64,
    /// Pico de chamadas simultâneas
    pub peak_concurrent_calls: u32,
    /// Taxa de atendimento (ASR)
    pub answer_seizure_ratio: f64,
    /// Duração média das chamadas (segundos)
    pub average_call_duration: f64,
    /// Uso de storage para gravações (MB)
    pub recording_storage_used: u64,
    /// Custos por categoria
    pub costs: UsageCosts,
}

/// Custos por categoria de uso
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UsageCosts {
    /// Custo de chamadas locais
    pub local_calls: f64,
    /// Custo de chamadas nacionais
    pub national_calls: f64,
    /// Custo de chamadas internacionais
    pub international_calls: f64,
    /// Custo de storage
    pub storage: f64,
    /// Custo total
    pub total: f64,
    /// Moeda
    pub currency: String,
}

impl Tenant {
    /// Cria novo tenant com configurações padrão
    pub fn new(request: CreateTenantRequest) -> Self {
        let limits = request.custom_limits
            .unwrap_or_else(|| TenantLimits::from_service_plan(&request.service_plan));

        let config = TenantConfig {
            sip_domain: request.sip_domain,
            default_context: format!("{}_context", request.id),
            default_gateway: request.config.as_ref()
                .and_then(|c| c.default_gateway.clone()),
            recording_policy: request.config.as_ref()
                .and_then(|c| c.recording_policy.clone())
                .unwrap_or(RecordingPolicy::Never),
            presence_enabled: request.config.as_ref()
                .and_then(|c| c.presence_enabled)
                .unwrap_or(true),
            conference_settings: request.config.as_ref()
                .and_then(|c| c.conference_settings.clone())
                .unwrap_or_default(),
            ivr_settings: request.config.as_ref()
                .and_then(|c| c.ivr_settings.clone())
                .unwrap_or_default(),
            timezone: request.config.as_ref()
                .and_then(|c| c.timezone.clone())
                .unwrap_or_else(|| "America/Sao_Paulo".to_string()),
            billing_settings: request.config.as_ref()
                .and_then(|c| c.billing_settings.clone())
                .unwrap_or_default(),
        };

        Self {
            id: request.id,
            name: request.name,
            status: TenantStatus::Active,
            service_plan: request.service_plan,
            limits,
            config,
            contact: request.contact,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Atualiza tenant com nova informação
    pub fn update(&mut self, request: UpdateTenantRequest) {
        if let Some(name) = request.name {
            self.name = name;
        }
        
        if let Some(status) = request.status {
            self.status = status;
        }
        
        if let Some(service_plan) = request.service_plan {
            self.service_plan = service_plan;
            // Atualizar limites se não foram customizados
            if request.limits.is_none() {
                self.limits = TenantLimits::from_service_plan(&self.service_plan);
            }
        }
        
        if let Some(limits) = request.limits {
            self.limits = limits;
        }
        
        if let Some(config_req) = request.config {
            self.apply_config_update(config_req);
        }
        
        if let Some(contact) = request.contact {
            self.contact = contact;
        }
        
        if let Some(expires_at) = request.expires_at {
            self.expires_at = Some(expires_at);
        }
        
        self.updated_at = Utc::now();
    }

    /// Aplica atualizações de configuração
    fn apply_config_update(&mut self, config_req: TenantConfigRequest) {
        if let Some(gateway) = config_req.default_gateway {
            self.config.default_gateway = Some(gateway);
        }
        
        if let Some(policy) = config_req.recording_policy {
            self.config.recording_policy = policy;
        }
        
        if let Some(presence) = config_req.presence_enabled {
            self.config.presence_enabled = presence;
        }
        
        if let Some(conf_settings) = config_req.conference_settings {
            self.config.conference_settings = conf_settings;
        }
        
        if let Some(ivr_settings) = config_req.ivr_settings {
            self.config.ivr_settings = ivr_settings;
        }
        
        if let Some(timezone) = config_req.timezone {
            self.config.timezone = timezone;
        }
        
        if let Some(billing) = config_req.billing_settings {
            self.config.billing_settings = billing;
        }
    }

    /// Verifica se tenant está ativo
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active || self.status == TenantStatus::Trial
    }

    /// Verifica se tenant pode fazer chamadas
    pub fn can_make_calls(&self) -> bool {
        self.is_active()
    }

    /// Verifica se tenant expirou (para trials)
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Gera resumo do tenant
    pub fn to_summary(&self, active_extensions: u32, active_calls: u32) -> TenantSummary {
        TenantSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            status: self.status.clone(),
            service_plan: self.service_plan.clone(),
            sip_domain: self.config.sip_domain.clone(),
            active_extensions,
            active_calls,
            created_at: self.created_at,
        }
    }

    /// Valida se recurso está dentro dos limites
    pub fn validate_limits(&self, usage: &TenantUsage) -> Result<(), String> {
        if usage.total_calls > self.limits.max_concurrent_calls as u64 {
            return Err("Concurrent call limit exceeded".to_string());
        }
        
        if usage.total_minutes > self.limits.max_monthly_minutes as u64 {
            return Err("Monthly minutes limit exceeded".to_string());
        }
        
        if usage.recording_storage_used > self.limits.max_recording_storage_mb {
            return Err("Recording storage limit exceeded".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_creation() {
        let request = CreateTenantRequest {
            id: "test_tenant".to_string(),
            name: "Test Company".to_string(),
            service_plan: ServicePlan::Professional,
            sip_domain: "test.pbx.com".to_string(),
            contact: TenantContact {
                company_name: "Test Company".to_string(),
                contact_name: "John Doe".to_string(),
                email: "john@test.com".to_string(),
                phone: "+5511999999999".to_string(),
                address: None,
                city: None,
                state: None,
                postal_code: None,
                country: "Brazil".to_string(),
            },
            config: None,
            custom_limits: None,
        };

        let tenant = Tenant::new(request);
        
        assert_eq!(tenant.id, "test_tenant");
        assert_eq!(tenant.name, "Test Company");
        assert_eq!(tenant.service_plan, ServicePlan::Professional);
        assert!(tenant.is_active());
        assert!(!tenant.is_expired());
    }

    #[test]
    fn test_service_plan_limits() {
        let basic_limits = TenantLimits::from_service_plan(&ServicePlan::Basic);
        assert_eq!(basic_limits.max_concurrent_calls, 5);
        
        let enterprise_limits = TenantLimits::from_service_plan(&ServicePlan::Enterprise);
        assert_eq!(enterprise_limits.max_concurrent_calls, 100);
    }
}

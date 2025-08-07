//! 📞 SIP User Models - Usuários e extensões SIP
//! 
//! Este módulo define os modelos para usuários SIP, extensões e 
//! configurações necessárias para integração com softphones como MicroSIP.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Usuário SIP com extensão e credenciais
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SipUser {
    /// ID único do usuário
    pub id: Uuid,
    /// ID do tenant ao qual pertence
    pub tenant_id: String,
    /// Número da extensão (ex: 1001, 1002)
    pub extension: String,
    /// Nome de exibição do usuário
    pub display_name: String,
    /// Email do usuário (opcional)
    pub email: Option<String>,
    /// Senha SIP (criptografada)
    pub password_hash: String,
    /// Status do usuário
    pub status: SipUserStatus,
    /// Configurações específicas do usuário
    pub config: SipUserConfig,
    /// Limites do usuário
    pub limits: SipUserLimits,
    /// Última vez que se registrou
    pub last_registration: Option<DateTime<Utc>>,
    /// IP do último registro
    pub last_ip: Option<String>,
    /// User-Agent do último registro
    pub last_user_agent: Option<String>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Data de atualização
    pub updated_at: DateTime<Utc>,
}

/// Status do usuário SIP
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SipUserStatus {
    /// Usuário ativo e pode se registrar
    Active,
    /// Usuário desativado temporariamente
    Inactive,
    /// Usuário suspenso por violação de políticas
    Suspended,
    /// Usuário bloqueado por excesso de tentativas
    Blocked,
    /// Usuário pendente de ativação
    Pending,
}

/// Configurações específicas do usuário SIP
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SipUserConfig {
    /// Codec de áudio preferido
    pub preferred_codec: String,
    /// Permitir chamadas externas
    pub allow_external_calls: bool,
    /// Permitir transferências
    pub allow_transfers: bool,
    /// Permitir conferências
    pub allow_conferences: bool,
    /// Gravar chamadas por padrão
    pub auto_record: bool,
    /// Timeout de registro (segundos)
    pub registration_timeout: u32,
    /// Usar SRTP (segurança)
    pub force_srtp: bool,
    /// Context personalizado (se diferente do tenant)
    pub custom_context: Option<String>,
    /// Variables customizadas
    pub custom_variables: std::collections::HashMap<String, String>,
}

/// Limites específicos do usuário
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SipUserLimits {
    /// Número máximo de chamadas simultâneas
    pub max_concurrent_calls: u32,
    /// Duração máxima de chamada (segundos)
    pub max_call_duration: Option<u32>,
    /// Minutos mensais permitidos
    pub monthly_minutes_limit: Option<u32>,
    /// Permitir chamadas internacionais
    pub allow_international: bool,
    /// Permitir chamadas de longa distância
    pub allow_long_distance: bool,
}

/// Request para criar usuário SIP
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSipUserRequest {
    /// Número da extensão
    pub extension: String,
    /// Nome de exibição
    pub display_name: String,
    /// Email (opcional)
    pub email: Option<String>,
    /// Senha em texto claro (será criptografada)
    pub password: String,
    /// Configurações opcionais (usa defaults se não fornecido)
    pub config: Option<SipUserConfig>,
    /// Limites opcionais (usa defaults se não fornecido)
    pub limits: Option<SipUserLimits>,
}

/// Request para atualizar usuário SIP
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSipUserRequest {
    /// Novo nome de exibição
    pub display_name: Option<String>,
    /// Novo email
    pub email: Option<String>,
    /// Nova senha (se fornecida)
    pub password: Option<String>,
    /// Novo status
    pub status: Option<SipUserStatus>,
    /// Novas configurações
    pub config: Option<SipUserConfig>,
    /// Novos limites
    pub limits: Option<SipUserLimits>,
}

/// Response com informações do usuário (sem senha)
#[derive(Debug, Serialize, ToSchema)]
pub struct SipUserResponse {
    /// ID único do usuário
    pub id: Uuid,
    /// ID do tenant
    pub tenant_id: String,
    /// Número da extensão
    pub extension: String,
    /// Nome de exibição
    pub display_name: String,
    /// Email
    pub email: Option<String>,
    /// Status
    pub status: SipUserStatus,
    /// Se está registrado atualmente
    pub is_registered: bool,
    /// Última vez que se registrou
    pub last_registration: Option<DateTime<Utc>>,
    /// IP do último registro
    pub last_ip: Option<String>,
    /// User-Agent do último registro
    pub last_user_agent: Option<String>,
    /// Configurações
    pub config: SipUserConfig,
    /// Limites
    pub limits: SipUserLimits,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Data de atualização
    pub updated_at: DateTime<Utc>,
}

/// Lista paginada de usuários SIP
#[derive(Debug, Serialize, ToSchema)]
pub struct SipUserListResponse {
    /// Lista de usuários
    pub users: Vec<SipUserResponse>,
    /// Total de usuários
    pub total: u64,
    /// Página atual
    pub page: u32,
    /// Itens por página
    pub per_page: u32,
    /// Total de páginas
    pub total_pages: u32,
}

/// Informações de registro SIP em tempo real
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SipRegistrationInfo {
    /// ID do usuário
    pub user_id: Uuid,
    /// Extensão
    pub extension: String,
    /// Se está registrado
    pub is_registered: bool,
    /// Contato SIP
    pub contact: Option<String>,
    /// Tempo de expiração do registro (segundos)
    pub expires: Option<u32>,
    /// User-Agent
    pub user_agent: Option<String>,
    /// IP de origem
    pub ip_address: Option<String>,
    /// Timestamp do registro
    pub registered_at: Option<DateTime<Utc>>,
}

/// Estatísticas de uso do usuário SIP
#[derive(Debug, Serialize, ToSchema)]
pub struct SipUserUsageStats {
    /// ID do usuário
    pub user_id: Uuid,
    /// Extensão
    pub extension: String,
    /// Total de chamadas no período
    pub total_calls: u64,
    /// Chamadas atendidas
    pub answered_calls: u64,
    /// Chamadas perdidas
    pub missed_calls: u64,
    /// Minutos falados no período
    pub total_minutes: u64,
    /// Chamadas simultâneas máximas
    pub peak_concurrent_calls: u32,
    /// Início do período de estatísticas
    pub period_start: DateTime<Utc>,
    /// Fim do período de estatísticas
    pub period_end: DateTime<Utc>,
}

// ==================== IMPLEMENTAÇÕES ====================

impl Default for SipUserConfig {
    fn default() -> Self {
        Self {
            preferred_codec: "PCMU".to_string(),
            allow_external_calls: false,
            allow_transfers: true,
            allow_conferences: true,
            auto_record: false,
            registration_timeout: 3600, // 1 hora
            force_srtp: false,
            custom_context: None,
            custom_variables: std::collections::HashMap::new(),
        }
    }
}

impl Default for SipUserLimits {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 2,
            max_call_duration: None,
            monthly_minutes_limit: None,
            allow_international: false,
            allow_long_distance: false,
        }
    }
}

impl SipUser {
    /// Cria novo usuário SIP
    pub fn new(tenant_id: String, request: CreateSipUserRequest, password_hash: String) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            extension: request.extension,
            display_name: request.display_name,
            email: request.email,
            password_hash,
            status: SipUserStatus::Active,
            config: request.config.unwrap_or_default(),
            limits: request.limits.unwrap_or_default(),
            last_registration: None,
            last_ip: None,
            last_user_agent: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Converte para response (sem senha)
    pub fn to_response(&self, is_registered: bool) -> SipUserResponse {
        SipUserResponse {
            id: self.id,
            tenant_id: self.tenant_id.clone(),
            extension: self.extension.clone(),
            display_name: self.display_name.clone(),
            email: self.email.clone(),
            status: self.status.clone(),
            is_registered,
            last_registration: self.last_registration,
            last_ip: self.last_ip.clone(),
            last_user_agent: self.last_user_agent.clone(),
            config: self.config.clone(),
            limits: self.limits.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    /// Atualiza último registro
    pub fn update_last_registration(&mut self, ip: Option<String>, user_agent: Option<String>) {
        self.last_registration = Some(Utc::now());
        self.last_ip = ip;
        self.last_user_agent = user_agent;
        self.updated_at = Utc::now();
    }

    /// Verifica se pode fazer chamadas externas
    pub fn can_make_external_calls(&self) -> bool {
        self.status == SipUserStatus::Active && self.config.allow_external_calls
    }

    /// Verifica se pode fazer transferências
    pub fn can_transfer_calls(&self) -> bool {
        self.status == SipUserStatus::Active && self.config.allow_transfers
    }

    /// Gera URI SIP completo
    pub fn sip_uri(&self, domain: &str) -> String {
        format!("sip:{}@{}", self.extension, domain)
    }
}

impl Default for SipUserStatus {
    fn default() -> Self {
        SipUserStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sip_user() {
        let request = CreateSipUserRequest {
            extension: "1001".to_string(),
            display_name: "João Silva".to_string(),
            email: Some("joao@empresa.com".to_string()),
            password: "senha123".to_string(),
            config: None,
            limits: None,
        };

        let user = SipUser::new(
            "demo_company".to_string(),
            request,
            "hashed_password".to_string(),
        );

        assert_eq!(user.extension, "1001");
        assert_eq!(user.display_name, "João Silva");
        assert_eq!(user.tenant_id, "demo_company");
        assert_eq!(user.status, SipUserStatus::Active);
    }

    #[test]
    fn test_sip_uri_generation() {
        let request = CreateSipUserRequest {
            extension: "1001".to_string(),
            display_name: "Test User".to_string(),
            email: None,
            password: "test".to_string(),
            config: None,
            limits: None,
        };

        let user = SipUser::new("tenant1".to_string(), request, "hash".to_string());
        let uri = user.sip_uri("demo.pbx.local");
        
        assert_eq!(uri, "sip:1001@demo.pbx.local");
    }

    #[test]
    fn test_permissions() {
        let mut request = CreateSipUserRequest {
            extension: "1001".to_string(),
            display_name: "Test User".to_string(),
            email: None,
            password: "test".to_string(),
            config: Some(SipUserConfig {
                allow_external_calls: true,
                allow_transfers: false,
                ..Default::default()
            }),
            limits: None,
        };

        let user = SipUser::new("tenant1".to_string(), request, "hash".to_string());
        
        assert!(user.can_make_external_calls());
        assert!(!user.can_transfer_calls());
    }
}

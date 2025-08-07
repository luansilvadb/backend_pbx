//! 🔧 Tipos comuns ESL - Estruturas compartilhadas e utilitários
//! 
//! Este módulo contém tipos, constantes e utilitários compartilhados
//! entre os módulos de eventos e comandos ESL.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Configuração para conexão ESL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EslConnectionConfig {
    /// Host do FreeSWITCH
    pub host: String,
    /// Porta ESL (geralmente 8021)
    pub port: u16,
    /// Senha de autenticação
    pub password: String,
    /// Timeout para operações (segundos)
    pub timeout_seconds: u32,
    /// Número máximo de tentativas de reconexão
    pub max_reconnect_attempts: u32,
    /// Intervalo entre tentativas de reconexão (segundos)
    pub reconnect_interval_seconds: u32,
}

impl Default for EslConnectionConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8021,
            password: "ClueCon".to_string(),
            timeout_seconds: 30,
            max_reconnect_attempts: 10,
            reconnect_interval_seconds: 5,
        }
    }
}

/// Métricas de performance do ESL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EslMetrics {
    /// Número total de eventos processados
    pub events_processed: u64,
    /// Número total de comandos executados
    pub commands_executed: u64,
    /// Número de erros de conexão
    pub connection_errors: u64,
    /// Número de reconexões realizadas
    pub reconnections: u64,
    /// Latência média de comandos (ms)
    pub average_command_latency_ms: f64,
    /// Eventos por segundo (média móvel)
    pub events_per_second: f64,
    /// Timestamp da última atualização
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for EslMetrics {
    fn default() -> Self {
        Self {
            events_processed: 0,
            commands_executed: 0,
            connection_errors: 0,
            reconnections: 0,
            average_command_latency_ms: 0.0,
            events_per_second: 0.0,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Estado da conexão ESL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EslConnectionState {
    /// Desconectado
    Disconnected,
    /// Conectando
    Connecting,
    /// Autenticando
    Authenticating,
    /// Conectado e pronto
    Connected,
    /// Erro de conexão
    Error(String),
    /// Reconectando após erro
    Reconnecting,
}

/// Prioridade de um comando ESL
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    /// Comando de sistema crítico (shutdown, etc.)
    Critical = 0,
    /// Comando urgente relacionado a chamadas ativas
    High = 1,
    /// Comando normal de operação
    Normal = 2,
    /// Comando de baixa prioridade (métricas, logs)
    Low = 3,
}

/// Contexto de execução de um comando
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// ID do tenant que está executando
    pub tenant_id: String,
    /// UUID da chamada relacionada (se aplicável)
    pub call_uuid: Option<Uuid>,
    /// Prioridade do comando
    pub priority: CommandPriority,
    /// Metadados adicionais
    pub metadata: HashMap<String, String>,
    /// Timestamp de criação
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CommandContext {
    pub fn new(tenant_id: String) -> Self {
        Self {
            tenant_id,
            call_uuid: None,
            priority: CommandPriority::Normal,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }
    
    pub fn with_call(mut self, call_uuid: Uuid) -> Self {
        self.call_uuid = Some(call_uuid);
        self
    }
    
    pub fn with_priority(mut self, priority: CommandPriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Filtro para pesquisa de chamadas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFilter {
    /// Filtrar por tenant específico
    pub tenant_id: Option<String>,
    /// Filtrar por status específico
    pub status: Option<String>,
    /// Filtrar por número de origem
    pub caller_id: Option<String>,
    /// Filtrar por número de destino
    pub destination: Option<String>,
    /// Filtrar por período (início)
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    /// Filtrar por período (fim)
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
    /// Limite de resultados
    pub limit: Option<u32>,
    /// Offset para paginação
    pub offset: Option<u32>,
}

impl Default for CallFilter {
    fn default() -> Self {
        Self {
            tenant_id: None,
            status: None,
            caller_id: None,
            destination: None,
            created_after: None,
            created_before: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

/// Resultado paginado de consultas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    /// Itens da página atual
    pub items: Vec<T>,
    /// Total de itens disponíveis
    pub total: u64,
    /// Página atual (baseada em 1)
    pub page: u32,
    /// Itens por página
    pub per_page: u32,
    /// Total de páginas
    pub total_pages: u32,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, per_page: u32) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

/// Informações de um tenant para validação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    /// ID único do tenant
    pub id: String,
    /// Nome da empresa/organização
    pub name: String,
    /// Domínio SIP associado
    pub sip_domain: String,
    /// Context padrão no dialplan
    pub default_context: String,
    /// Limites de recurso
    pub limits: TenantLimits,
    /// Status do tenant
    pub status: TenantStatus,
}

/// Limites de recursos por tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantLimits {
    /// Máximo de chamadas simultâneas
    pub max_concurrent_calls: u32,
    /// Máximo de ramais
    pub max_extensions: u32,
    /// Máximo de minutos por mês
    pub max_monthly_minutes: u32,
    /// Tamanho máximo de gravações (MB)
    pub max_recording_storage_mb: u64,
}

impl Default for TenantLimits {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 10,
            max_extensions: 50,
            max_monthly_minutes: 1000,
            max_recording_storage_mb: 1024,
        }
    }
}

/// Status de um tenant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TenantStatus {
    /// Ativo e operacional
    Active,
    /// Suspenso temporariamente
    Suspended,
    /// Cancelado/inativo
    Inactive,
    /// Em período de teste
    Trial,
}

/// Validador de isolamento multi-tenant
pub struct TenantValidator;

impl TenantValidator {
    /// Valida se um recurso pertence ao tenant
    pub fn validate_resource_access(tenant_id: &str, resource_path: &str) -> Result<(), String> {
        // Verificar se path contém ID do tenant
        if !resource_path.contains(&format!("/{}/", tenant_id)) && 
           !resource_path.contains(&format!("{}.", tenant_id)) {
            return Err(format!("Resource {} does not belong to tenant {}", resource_path, tenant_id));
        }
        
        Ok(())
    }
    
    /// Valida se um UUID de chamada pertence ao tenant
    pub fn validate_call_access(tenant_id: &str, call_uuid: &Uuid, tenant_context: &HashMap<String, String>) -> Result<(), String> {
        if let Some(call_tenant) = tenant_context.get("tenant_id") {
            if call_tenant != tenant_id {
                return Err(format!("Call {} does not belong to tenant {}", call_uuid, tenant_id));
            }
        } else {
            return Err("Call does not have tenant context".to_string());
        }
        
        Ok(())
    }
    
    /// Valida se domínio SIP pertence ao tenant
    pub fn validate_sip_domain(tenant_id: &str, domain: &str) -> Result<(), String> {
        if domain != tenant_id && !domain.ends_with(&format!(".{}", tenant_id)) {
            return Err(format!("SIP domain {} does not belong to tenant {}", domain, tenant_id));
        }
        
        Ok(())
    }
}

/// Utilitários para parsing de headers ESL
pub struct EslHeaderParser;

impl EslHeaderParser {
    /// Extrai UUID de chamada de headers
    pub fn extract_call_uuid(headers: &HashMap<String, String>) -> Option<Uuid> {
        headers.get("Unique-ID")
            .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok())
    }
    
    /// Extrai ID do tenant de headers
    pub fn extract_tenant_id(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("variable_tenant_id")
            .or_else(|| headers.get("variable_domain"))
            .or_else(|| headers.get("realm"))
            .cloned()
    }
    
    /// Converte valor de header para número
    pub fn parse_numeric_header<T: std::str::FromStr>(headers: &HashMap<String, String>, key: &str) -> Option<T> {
        headers.get(key)
            .and_then(|value| value.parse().ok())
    }
    
    /// Converte timestamp de header para DateTime
    pub fn parse_timestamp_header(headers: &HashMap<String, String>, key: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        headers.get(key)
            .and_then(|timestamp_str| {
                // FreeSWITCH usa microsegundos desde epoch
                if let Ok(microseconds) = timestamp_str.parse::<i64>() {
                    chrono::DateTime::from_timestamp(microseconds / 1_000_000, 0)
                } else {
                    None
                }
            })
    }
}

/// Constantes do sistema ESL
pub mod constants {
    /// Eventos que sempre devem ser processados independente de filtros
    pub const CRITICAL_EVENTS: &[&str] = &[
        "SHUTDOWN",
        "TRAP", 
        "HEARTBEAT",
        "MODULE_LOAD",
        "MODULE_UNLOAD"
    ];
    
    /// Headers que contêm informações de tenant
    pub const TENANT_HEADERS: &[&str] = &[
        "variable_tenant_id",
        "variable_domain", 
        "realm",
        "Caller-Context"
    ];
    
    /// Variáveis de canal que devem ser sempre exportadas
    pub const EXPORT_VARIABLES: &[&str] = &[
        "tenant_id",
        "domain",
        "billing_account",
        "recording_policy"
    ];
    
    /// Timeout padrão para comandos ESL (segundos)
    pub const DEFAULT_COMMAND_TIMEOUT: u64 = 30;
    
    /// Tamanho do buffer de eventos
    pub const EVENT_BUFFER_SIZE: usize = 10000;
    
    /// Intervalo de heartbeat (segundos)
    pub const HEARTBEAT_INTERVAL: u64 = 60;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_validator() {
        let tenant_id = "tenant123";
        
        // Teste válido
        assert!(TenantValidator::validate_resource_access(tenant_id, "/files/tenant123/audio.wav").is_ok());
        
        // Teste inválido
        assert!(TenantValidator::validate_resource_access(tenant_id, "/files/tenant456/audio.wav").is_err());
        
        // Teste domínio válido
        assert!(TenantValidator::validate_sip_domain(tenant_id, "tenant123").is_ok());
        assert!(TenantValidator::validate_sip_domain(tenant_id, "subdomain.tenant123").is_ok());
        
        // Teste domínio inválido
        assert!(TenantValidator::validate_sip_domain(tenant_id, "tenant456").is_err());
    }

    #[test]
    fn test_esl_header_parser() {
        let mut headers = HashMap::new();
        headers.insert("Unique-ID".to_string(), "123e4567-e89b-12d3-a456-426614174000".to_string());
        headers.insert("variable_tenant_id".to_string(), "tenant123".to_string());
        headers.insert("variable_duration".to_string(), "60".to_string());
        
        assert!(EslHeaderParser::extract_call_uuid(&headers).is_some());
        assert_eq!(EslHeaderParser::extract_tenant_id(&headers), Some("tenant123".to_string()));
        assert_eq!(EslHeaderParser::parse_numeric_header::<u64>(&headers, "variable_duration"), Some(60));
    }
}

//! 📨 Eventos do FreeSWITCH - Definições completas para sistema multi-tenant
//! 
//! Este módulo define todos os eventos enviados pelo FreeSWITCH para o backend,
//! organizados por categoria e com suporte a multi-tenancy.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Tipo de evento do FreeSWITCH
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EslEventType {
    // ========== EVENTOS DE CICLO DE VIDA DA CHAMADA ==========
    /// Canal criado - primeiro ponto de captura para associação ao tenant
    ChannelCreate,
    /// Progresso da chamada (equivalente SIP 183)
    ChannelProgress,
    /// Chamada atendida - início do billing
    ChannelAnswer,
    /// Chamada encerrada - finalização do billing
    ChannelHangup,
    /// Canal removido da memória - limpeza final
    ChannelDestroy,
    
    // ========== EVENTOS DE INTERAÇÃO DO USUÁRIO ==========
    /// Dígitos DTMF pressionados pelo usuário
    Dtmf,
    /// Reprodução de áudio iniciada
    PlaybackStart,
    /// Reprodução de áudio finalizada
    PlaybackStop,
    /// Gravação iniciada
    RecordStart,
    /// Gravação finalizada
    RecordStop,
    
    // ========== EVENTOS DE STATUS E MANUTENÇÃO ==========
    /// Sinal de vida do FreeSWITCH
    Heartbeat,
    /// Configurações XML recarregadas
    ReloadXml,
    /// Módulo carregado
    ModuleLoad,
    /// Módulo descarregado
    ModuleUnload,
    /// Sistema sendo desligado
    Shutdown,
    /// Erro crítico ou crash
    Trap,
    
    // ========== EVENTOS DE REGISTRO E PRESENÇA ==========
    /// Endpoint SIP registrado
    SofiaRegister,
    /// Endpoint SIP desregistrado
    SofiaUnregister,
    /// Registro SIP expirou
    SofiaExpire,
    /// Estado de presença atualizado (entrada)
    PresenceIn,
    /// Estado de presença atualizado (saída)
    PresenceOut,
    
    // ========== EVENTOS PERSONALIZADOS E OUTROS ==========
    /// Evento personalizado definido pelo usuário
    Custom(String),
    /// Evento não reconhecido
    Unknown(String),
}

impl From<&str> for EslEventType {
    fn from(event_name: &str) -> Self {
        match event_name.to_uppercase().as_str() {
            // Ciclo de vida
            "CHANNEL_CREATE" => Self::ChannelCreate,
            "CHANNEL_PROGRESS" => Self::ChannelProgress,
            "CHANNEL_ANSWER" => Self::ChannelAnswer,
            "CHANNEL_HANGUP" => Self::ChannelHangup,
            "CHANNEL_DESTROY" => Self::ChannelDestroy,
            
            // Interação do usuário
            "DTMF" => Self::Dtmf,
            "PLAYBACK_START" => Self::PlaybackStart,
            "PLAYBACK_STOP" => Self::PlaybackStop,
            "RECORD_START" => Self::RecordStart,
            "RECORD_STOP" => Self::RecordStop,
            
            // Sistema
            "HEARTBEAT" => Self::Heartbeat,
            "RELOADXML" => Self::ReloadXml,
            "MODULE_LOAD" => Self::ModuleLoad,
            "MODULE_UNLOAD" => Self::ModuleUnload,
            "SHUTDOWN" => Self::Shutdown,
            "TRAP" => Self::Trap,
            
            // Registro e presença
            "SOFIA::REGISTER" => Self::SofiaRegister,
            "SOFIA::UNREGISTER" => Self::SofiaUnregister,
            "SOFIA::EXPIRE" => Self::SofiaExpire,
            "PRESENCE_IN" => Self::PresenceIn,
            "PRESENCE_OUT" => Self::PresenceOut,
            
            // Personalizado
            event if event.starts_with("CUSTOM") => {
                Self::Custom(event.to_string())
            }
            
            // Desconhecido
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

/// Estado de presença de um endpoint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PresenceState {
    /// Disponível para chamadas
    Available,
    /// Ocupado em chamada
    Busy,
    /// Ausente temporariamente
    Away,
    /// Não perturbe
    DoNotDisturb,
    /// Offline/desconectado
    Offline,
    /// Estado desconhecido
    Unknown,
}

impl From<&str> for PresenceState {
    fn from(state: &str) -> Self {
        match state.to_lowercase().as_str() {
            "available" | "online" => Self::Available,
            "busy" | "on-the-phone" => Self::Busy,
            "away" | "be-right-back" => Self::Away,
            "dnd" | "do-not-disturb" => Self::DoNotDisturb,
            "offline" | "unavailable" => Self::Offline,
            _ => Self::Unknown,
        }
    }
}

/// Código de desconexão da chamada (padrão ITU-T Q.850)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HangupCause {
    /// Desconexão normal
    NormalClearing,
    /// Usuário ocupado
    UserBusy,
    /// Não há resposta
    NoAnswer,
    /// Rejeitada pelo usuário
    CallRejected,
    /// Número inválido
    InvalidNumberFormat,
    /// Destino fora de serviço
    DestinationOutOfOrder,
    /// Erro de protocolo
    ProtocolError,
    /// Timeout de originating
    OriginatorCancel,
    /// Sistema ocupado
    SystemShutdown,
    /// Erro desconhecido
    Unknown(String),
}

impl From<&str> for HangupCause {
    fn from(cause: &str) -> Self {
        match cause.to_uppercase().as_str() {
            "NORMAL_CLEARING" => Self::NormalClearing,
            "USER_BUSY" => Self::UserBusy,
            "NO_ANSWER" => Self::NoAnswer,
            "CALL_REJECTED" => Self::CallRejected,
            "INVALID_NUMBER_FORMAT" => Self::InvalidNumberFormat,
            "DESTINATION_OUT_OF_ORDER" => Self::DestinationOutOfOrder,
            "PROTOCOL_ERROR" => Self::ProtocolError,
            "ORIGINATOR_CANCEL" => Self::OriginatorCancel,
            "SYSTEM_SHUTDOWN" => Self::SystemShutdown,
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

/// Informações específicas de tenant extraídas do evento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    /// ID do tenant proprietário da chamada
    pub tenant_id: Option<String>,
    /// Domínio SIP do tenant
    pub domain: Option<String>,
    /// Context do dialplan específico do tenant
    pub context: Option<String>,
    /// Identificação do gateway/provider usado
    pub gateway: Option<String>,
}

/// Estrutura completa de um evento ESL com contexto de tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EslEvent {
    /// Tipo do evento
    pub event_type: EslEventType,
    /// ID único da chamada (se aplicável)
    pub call_uuid: Option<Uuid>,
    /// Timestamp de recebimento do evento
    pub timestamp: DateTime<Utc>,
    /// Contexto do tenant
    pub tenant_context: TenantContext,
    /// Dados específicos do evento
    pub data: EslEventData,
    /// Headers ESL originais (para debug)
    pub raw_headers: HashMap<String, String>,
}

/// Dados específicos por tipo de evento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EslEventData {
    /// Dados de criação de canal
    ChannelCreate {
        caller_id_number: String,
        caller_id_name: String,
        destination_number: String,
        channel_name: String,
        context: String,
    },
    
    /// Dados de progresso da chamada
    ChannelProgress {
        caller_id_number: String,
        destination_number: String,
        progress_time: Option<u64>,
    },
    
    /// Dados de atendimento
    ChannelAnswer {
        caller_id_number: String,
        destination_number: String,
        answer_time: DateTime<Utc>,
    },
    
    /// Dados de desconexão
    ChannelHangup {
        caller_id_number: String,
        destination_number: String,
        hangup_cause: HangupCause,
        duration_seconds: Option<u64>,
        billable_seconds: Option<u64>,
    },
    
    /// Dados de DTMF
    Dtmf {
        digit: char,
        duration_ms: Option<u64>,
        source: String,
    },
    
    /// Dados de reprodução
    Playback {
        file_path: String,
        status: String,
        samples: Option<u64>,
    },
    
    /// Dados de gravação
    Recording {
        file_path: String,
        status: String,
        duration_ms: Option<u64>,
        file_size_bytes: Option<u64>,
    },
    
    /// Dados de registro SIP
    SofiaRegistration {
        user: String,
        realm: String,
        contact: String,
        expires: Option<u64>,
        user_agent: Option<String>,
    },
    
    /// Dados de presença
    Presence {
        from: String,
        state: PresenceState,
        status_text: Option<String>,
    },
    
    /// Dados de sistema/heartbeat
    System {
        message: String,
        details: Option<HashMap<String, String>>,
    },
    
    /// Evento personalizado
    Custom {
        subclass: String,
        body: String,
        headers: HashMap<String, String>,
    },
    
    /// Dados genéricos para eventos não mapeados
    Generic {
        headers: HashMap<String, String>,
    },
}

impl EslEvent {
    /// Cria um novo evento a partir dos headers ESL brutos
    pub fn from_headers(headers: HashMap<String, String>) -> Self {
        let event_name = headers.get("Event-Name").unwrap_or(&"UNKNOWN".to_string()).clone();
        let event_type = EslEventType::from(event_name.as_str());
        
        let call_uuid = headers.get("Unique-ID")
            .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok());
        
        let tenant_context = TenantContext::from_headers(&headers);
        let data = EslEventData::from_headers(&event_type, &headers);
        
        Self {
            event_type,
            call_uuid,
            timestamp: Utc::now(),
            tenant_context,
            data,
            raw_headers: headers,
        }
    }
    
    /// Verifica se o evento pertence a um tenant específico
    pub fn belongs_to_tenant(&self, tenant_id: &str) -> bool {
        self.tenant_context.tenant_id
            .as_ref()
            .map(|id| id == tenant_id)
            .unwrap_or(false)
    }
    
    /// Extrai o ID do tenant do evento (se disponível)
    pub fn get_tenant_id(&self) -> Option<&str> {
        self.tenant_context.tenant_id.as_deref()
    }
    
    /// Verifica se é um evento crítico que afeta todos os tenants
    pub fn is_system_critical(&self) -> bool {
        matches!(self.event_type, 
            EslEventType::Shutdown | 
            EslEventType::Trap | 
            EslEventType::Heartbeat
        )
    }
}

impl TenantContext {
    /// Extrai contexto de tenant dos headers ESL
    fn from_headers(headers: &HashMap<String, String>) -> Self {
        Self {
            tenant_id: headers.get("variable_tenant_id").cloned(),
            domain: headers.get("variable_domain").cloned(),
            context: headers.get("Caller-Context").cloned(),
            gateway: headers.get("variable_sofia_gateway_name").cloned(),
        }
    }
}

impl EslEventData {
    /// Cria dados específicos do evento baseado no tipo e headers
    fn from_headers(event_type: &EslEventType, headers: &HashMap<String, String>) -> Self {
        match event_type {
            EslEventType::ChannelCreate => Self::ChannelCreate {
                caller_id_number: headers.get("Caller-Caller-ID-Number").unwrap_or(&"unknown".to_string()).clone(),
                caller_id_name: headers.get("Caller-Caller-ID-Name").unwrap_or(&"unknown".to_string()).clone(),
                destination_number: headers.get("Caller-Destination-Number").unwrap_or(&"unknown".to_string()).clone(),
                channel_name: headers.get("Channel-Name").unwrap_or(&"unknown".to_string()).clone(),
                context: headers.get("Caller-Context").unwrap_or(&"default".to_string()).clone(),
            },
            
            EslEventType::ChannelProgress => Self::ChannelProgress {
                caller_id_number: headers.get("Caller-Caller-ID-Number").unwrap_or(&"unknown".to_string()).clone(),
                destination_number: headers.get("Caller-Destination-Number").unwrap_or(&"unknown".to_string()).clone(),
                progress_time: headers.get("variable_progress_stamp").and_then(|s| s.parse().ok()),
            },
            
            EslEventType::ChannelAnswer => Self::ChannelAnswer {
                caller_id_number: headers.get("Caller-Caller-ID-Number").unwrap_or(&"unknown".to_string()).clone(),
                destination_number: headers.get("Caller-Destination-Number").unwrap_or(&"unknown".to_string()).clone(),
                answer_time: Utc::now(), // Em produção, usar timestamp do header
            },
            
            EslEventType::ChannelHangup => Self::ChannelHangup {
                caller_id_number: headers.get("Caller-Caller-ID-Number").unwrap_or(&"unknown".to_string()).clone(),
                destination_number: headers.get("Caller-Destination-Number").unwrap_or(&"unknown".to_string()).clone(),
                hangup_cause: headers.get("Hangup-Cause")
                    .map(|cause| HangupCause::from(cause.as_str()))
                    .unwrap_or(HangupCause::Unknown("no_cause".to_string())),
                duration_seconds: headers.get("variable_duration").and_then(|s| s.parse().ok()),
                billable_seconds: headers.get("variable_billsec").and_then(|s| s.parse().ok()),
            },
            
            EslEventType::Dtmf => Self::Dtmf {
                digit: headers.get("DTMF-Digit")
                    .and_then(|d| d.chars().next())
                    .unwrap_or('0'),
                duration_ms: headers.get("DTMF-Duration").and_then(|s| s.parse().ok()),
                source: headers.get("DTMF-Source").unwrap_or(&"unknown".to_string()).clone(),
            },
            
            EslEventType::PlaybackStart | EslEventType::PlaybackStop => Self::Playback {
                file_path: headers.get("Playback-File-Path").unwrap_or(&"unknown".to_string()).clone(),
                status: headers.get("Playback-Status").unwrap_or(&"unknown".to_string()).clone(),
                samples: headers.get("variable_playback_samples").and_then(|s| s.parse().ok()),
            },
            
            EslEventType::RecordStart | EslEventType::RecordStop => Self::Recording {
                file_path: headers.get("Record-File-Path").unwrap_or(&"unknown".to_string()).clone(),
                status: headers.get("Record-Status").unwrap_or(&"unknown".to_string()).clone(),
                duration_ms: headers.get("variable_record_ms").and_then(|s| s.parse().ok()),
                file_size_bytes: headers.get("variable_record_file_size").and_then(|s| s.parse().ok()),
            },
            
            EslEventType::SofiaRegister | EslEventType::SofiaUnregister | EslEventType::SofiaExpire => Self::SofiaRegistration {
                user: headers.get("from-user").unwrap_or(&"unknown".to_string()).clone(),
                realm: headers.get("realm").unwrap_or(&"unknown".to_string()).clone(),
                contact: headers.get("contact").unwrap_or(&"unknown".to_string()).clone(),
                expires: headers.get("expires").and_then(|s| s.parse().ok()),
                user_agent: headers.get("user-agent").cloned(),
            },
            
            EslEventType::PresenceIn | EslEventType::PresenceOut => Self::Presence {
                from: headers.get("from").unwrap_or(&"unknown".to_string()).clone(),
                state: headers.get("status")
                    .map(|s| PresenceState::from(s.as_str()))
                    .unwrap_or(PresenceState::Unknown),
                status_text: headers.get("rpid").cloned(),
            },
            
            EslEventType::Custom(subclass) => Self::Custom {
                subclass: subclass.clone(),
                body: headers.get("Content-Length")
                    .and_then(|_| headers.get("Body"))
                    .unwrap_or(&"".to_string()).clone(),
                headers: headers.clone(),
            },
            
            _ => Self::System {
                message: headers.get("Core-UUID").unwrap_or(&"system_event".to_string()).clone(),
                details: Some(headers.clone()),
            },
        }
    }
}

/// Filtros para subscrição de eventos específicos
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// Tipos de eventos para escutar
    pub event_types: Vec<EslEventType>,
    /// Filtrar apenas eventos de tenant específico
    pub tenant_id: Option<String>,
    /// Filtrar apenas eventos de chamadas específicas
    pub call_uuid: Option<Uuid>,
    /// Incluir eventos críticos de sistema independente dos filtros
    pub include_system_events: bool,
}

impl EventSubscription {
    /// Cria subscrição para todos os eventos
    pub fn all() -> Self {
        Self {
            event_types: vec![],  // Vazio = todos
            tenant_id: None,
            call_uuid: None,
            include_system_events: true,
        }
    }
    
    /// Cria subscrição apenas para eventos de tenant específico
    pub fn for_tenant(tenant_id: String) -> Self {
        Self {
            event_types: vec![],
            tenant_id: Some(tenant_id),
            call_uuid: None,
            include_system_events: true,
        }
    }
    
    /// Cria subscrição para chamada específica
    pub fn for_call(call_uuid: Uuid) -> Self {
        Self {
            event_types: vec![],
            tenant_id: None,
            call_uuid: Some(call_uuid),
            include_system_events: false,
        }
    }
    
    /// Verifica se um evento deve ser enviado baseado nesta subscrição
    pub fn matches(&self, event: &EslEvent) -> bool {
        // Sempre incluir eventos críticos de sistema se configurado
        if self.include_system_events && event.is_system_critical() {
            return true;
        }
        
        // Filtrar por tipo de evento se especificado
        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type) {
            return false;
        }
        
        // Filtrar por tenant se especificado
        if let Some(ref tenant_id) = self.tenant_id {
            if !event.belongs_to_tenant(tenant_id) {
                return false;
            }
        }
        
        // Filtrar por UUID da chamada se especificado
        if let Some(ref call_uuid) = self.call_uuid {
            if event.call_uuid.as_ref() != Some(call_uuid) {
                return false;
            }
        }
        
        true
    }
}

//! 📞 Call Model - Estruturas de dados para chamadas

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Status de uma chamada
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CallStatus {
    /// Chamada sendo criada
    Creating,
    /// Chamada tocando
    Ringing,
    /// Chamada atendida
    Answered,
    /// Chamada conectada (bridge)
    Connected,
    /// Chamada encerrada
    Hangup,
    /// Chamada ocupada
    Busy,
    /// Chamada falhou
    Failed,
}

/// Informações completas de uma chamada
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Call {
    /// ID único da chamada
    pub id: Uuid,
    /// Número/ramal de origem
    pub from: String,
    /// Número/ramal de destino  
    pub to: String,
    /// Status atual da chamada
    pub status: CallStatus,
    /// Timestamp de criação
    pub created_at: DateTime<Utc>,
    /// Timestamp de atendimento (se aplicável)
    pub answered_at: Option<DateTime<Utc>>,
    /// Timestamp de encerramento (se aplicável)
    pub ended_at: Option<DateTime<Utc>>,
    /// Contexto do dialplan
    pub context: Option<String>,
    /// Duração em segundos (calculada)
    pub duration_seconds: Option<u64>,
}

/// Request para originar uma nova chamada
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OriginateCallRequest {
    /// Número/ramal de origem
    pub from: String,
    /// Número/ramal de destino
    pub to: String,
    /// Timeout em segundos (opcional)
    pub timeout_seconds: Option<u32>,
    /// Contexto do dialplan (opcional)
    pub context: Option<String>,
}

/// Response para listagem de chamadas
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CallListResponse {
    /// Lista de chamadas
    pub calls: Vec<Call>,
    /// Total de chamadas ativas
    pub total: usize,
}

/// Resumo do status de uma chamada
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CallSummary {
    /// ID único da chamada
    pub id: Uuid,
    /// Número/ramal de origem
    pub from: String,
    /// Número/ramal de destino
    pub to: String,
    /// Status atual
    pub status: CallStatus,
    /// Duração em segundos (se aplicável)
    pub duration_seconds: Option<u64>,
}

impl Call {
    /// Cria uma nova chamada
    pub fn new(from: String, to: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            status: CallStatus::Creating,
            created_at: Utc::now(),
            answered_at: None,
            ended_at: None,
            context: None,
            duration_seconds: None,
        }
    }

    /// Atualiza o status da chamada
    pub fn update_status(&mut self, status: CallStatus) {
        self.status = status.clone();
        
        match status {
            CallStatus::Answered => {
                if self.answered_at.is_none() {
                    self.answered_at = Some(Utc::now());
                }
            }
            CallStatus::Hangup | CallStatus::Busy | CallStatus::Failed => {
                if self.ended_at.is_none() {
                    self.ended_at = Some(Utc::now());
                    self.calculate_duration();
                }
            }
            _ => {}
        }
    }

    /// Calcula a duração da chamada
    fn calculate_duration(&mut self) {
        if let (Some(answered), Some(ended)) = (self.answered_at, self.ended_at) {
            self.duration_seconds = Some((ended - answered).num_seconds() as u64);
        }
    }

    /// Converte para resumo
    pub fn to_summary(&self) -> CallSummary {
        CallSummary {
            id: self.id,
            from: self.from.clone(),
            to: self.to.clone(),
            status: self.status.clone(),
            duration_seconds: self.duration_seconds,
        }
    }
}

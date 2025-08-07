//! 📞 Call Manager - Gerenciamento de chamadas

use crate::models::{ApiError, ApiResult, Call, CallStatus, OriginateCallRequest};
use crate::services::{FreeSwitchEvent, FreeSwitchService};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Gerenciador de chamadas
pub struct CallManager {
    /// Serviço FreeSWITCH
    freeswitch: Arc<dyn FreeSwitchService>,
    /// Cache de chamadas ativas
    active_calls: Arc<RwLock<HashMap<Uuid, Call>>>,
}

impl CallManager {
    /// Cria nova instância do gerenciador
    pub fn new(freeswitch: Arc<dyn FreeSwitchService>) -> Self {
        Self {
            freeswitch,
            active_calls: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Inicia o gerenciador de chamadas
    pub async fn start(&self) -> ApiResult<()> {
        info!("Starting Call Manager");
        
        // Conectar ao FreeSWITCH
        self.freeswitch.connect().await?;
        
        // Iniciar processamento de eventos
        self.start_event_processing().await?;
        
        info!("Call Manager started successfully");
        Ok(())
    }

    /// Lista todas as chamadas ativas
    pub async fn list_active_calls(&self) -> Vec<Call> {
        let calls = self.active_calls.read().await;
        calls.values().cloned().collect()
    }

    /// Obtém uma chamada específica
    pub async fn get_call(&self, call_id: &Uuid) -> ApiResult<Call> {
        let calls = self.active_calls.read().await;
        calls.get(call_id)
            .cloned()
            .ok_or_else(|| ApiError::CallNotFound(*call_id))
    }

    /// Origina uma nova chamada
    pub async fn originate_call(&self, request: OriginateCallRequest) -> ApiResult<Call> {
        info!("Originating call from {} to {}", request.from, request.to);
        
        // Criar nova chamada
        let mut call = Call::new(request.from.clone(), request.to.clone());
        call.context = request.context.clone();
        
        // Originar via FreeSWITCH
        let fs_call_id = self.freeswitch
            .originate_call(&request.from, &request.to, request.timeout_seconds)
            .await?;
        
        // Usar o ID retornado pelo FreeSWITCH (se aplicável)
        call.id = fs_call_id;
        call.update_status(CallStatus::Ringing);
        
        // Adicionar ao cache
        let mut calls = self.active_calls.write().await;
        calls.insert(call.id, call.clone());
        
        info!("Call originated successfully: {}", call.id);
        Ok(call)
    }

    /// Encerra uma chamada
    pub async fn hangup_call(&self, call_id: &Uuid) -> ApiResult<()> {
        info!("Hanging up call: {}", call_id);
        
        // Verificar se a chamada existe
        let call_exists = {
            let calls = self.active_calls.read().await;
            calls.contains_key(call_id)
        };
        
        if !call_exists {
            return Err(ApiError::CallNotFound(*call_id));
        }
        
        // Encerrar via FreeSWITCH
        self.freeswitch.hangup_call(call_id).await?;
        
        // Atualizar status local
        self.update_call_status(call_id, CallStatus::Hangup).await;
        
        info!("Call hung up successfully: {}", call_id);
        Ok(())
    }

    /// Obtém estatísticas das chamadas
    pub async fn get_call_statistics(&self) -> HashMap<String, usize> {
        let calls = self.active_calls.read().await;
        let mut stats = HashMap::new();
        
        stats.insert("total_active".to_string(), calls.len());
        
        let mut by_status = HashMap::new();
        for call in calls.values() {
            let status = match call.status {
                CallStatus::Creating => "creating",
                CallStatus::Ringing => "ringing", 
                CallStatus::Answered => "answered",
                CallStatus::Connected => "connected",
                CallStatus::Hangup => "hangup",
                CallStatus::Busy => "busy",
                CallStatus::Failed => "failed",
            };
            *by_status.entry(status).or_insert(0) += 1;
        }
        
        for (status, count) in by_status {
            stats.insert(format!("status_{}", status), count);
        }
        
        stats
    }

    /// Inicia processamento de eventos do FreeSWITCH
    async fn start_event_processing(&self) -> ApiResult<()> {
        let mut event_receiver = self.freeswitch.subscribe_events().await?;
        let call_manager = Arc::new(self.clone());
        
        tokio::spawn(async move {
            info!("Starting FreeSWITCH event processing");
            
            loop {
                match event_receiver.recv().await {
                    Ok(event) => {
                        if let Err(e) = call_manager.handle_freeswitch_event(event).await {
                            warn!("Error handling FreeSWITCH event: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Error receiving FreeSWITCH event: {}", e);
                        break;
                    }
                }
            }
            
            warn!("FreeSWITCH event processing stopped");
        });
        
        Ok(())
    }

    /// Processa eventos do FreeSWITCH
    async fn handle_freeswitch_event(&self, event: FreeSwitchEvent) -> ApiResult<()> {
        debug!("Handling FreeSWITCH event: {}", event.event_type);
        
        match event.event_type.as_str() {
            "CHANNEL_CREATE" => {
                if let Some(call_id) = event.call_id {
                    self.handle_channel_create(call_id, &event).await?;
                }
            }
            "CHANNEL_ANSWER" => {
                if let Some(call_id) = event.call_id {
                    self.update_call_status(&call_id, CallStatus::Answered).await;
                }
            }
            "CHANNEL_BRIDGE" => {
                if let Some(call_id) = event.call_id {
                    self.update_call_status(&call_id, CallStatus::Connected).await;
                }
            }
            "CHANNEL_HANGUP" => {
                if let Some(call_id) = event.call_id {
                    self.handle_channel_hangup(call_id, &event).await;
                }
            }
            _ => {
                debug!("Unhandled event type: {}", event.event_type);
            }
        }
        
        Ok(())
    }

    /// Trata evento de criação de canal
    async fn handle_channel_create(&self, call_id: Uuid, event: &FreeSwitchEvent) -> ApiResult<()> {
        let from = event.data.get("Caller-Caller-ID-Number")
            .unwrap_or(&"unknown".to_string())
            .clone();
        let to = event.data.get("Caller-Destination-Number")
            .unwrap_or(&"unknown".to_string())
            .clone();
        
        let mut call = Call::new(from, to);
        call.id = call_id;
        call.update_status(CallStatus::Ringing);
        
        let mut calls = self.active_calls.write().await;
        calls.insert(call_id, call);
        
        info!("New call created: {}", call_id);
        Ok(())
    }

    /// Trata evento de encerramento de canal
    async fn handle_channel_hangup(&self, call_id: Uuid, _event: &FreeSwitchEvent) {
        self.update_call_status(&call_id, CallStatus::Hangup).await;
        
        // Remove da lista de chamadas ativas após um tempo
        let calls = Arc::clone(&self.active_calls);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let mut calls = calls.write().await;
            calls.remove(&call_id);
            debug!("Removed call from cache: {}", call_id);
        });
        
        info!("Call ended: {}", call_id);
    }

    /// Atualiza status de uma chamada
    async fn update_call_status(&self, call_id: &Uuid, status: CallStatus) {
        let mut calls = self.active_calls.write().await;
        if let Some(call) = calls.get_mut(call_id) {
            call.update_status(status);
            debug!("Updated call {} status to {:?}", call_id, call.status);
        }
    }
}

// Implementar Clone para usar em Arc
impl Clone for CallManager {
    fn clone(&self) -> Self {
        Self {
            freeswitch: Arc::clone(&self.freeswitch),
            active_calls: Arc::clone(&self.active_calls),
        }
    }
}

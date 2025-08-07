//! 📡 FreeSWITCH Service - Abstração da comunicação ESL

use crate::config::FreeSwitchConfig;
use crate::models::{ApiError, ApiResult};
use async_trait::async_trait;
use futures::stream::SplitSink;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Evento recebido do FreeSWITCH
#[derive(Debug, Clone)]
pub struct FreeSwitchEvent {
    pub event_type: String,
    pub call_id: Option<Uuid>,
    pub data: HashMap<String, String>,
    pub raw_data: String,
}

/// Trait para comunicação com FreeSWITCH
#[async_trait]
pub trait FreeSwitchService: Send + Sync {
    /// Conecta ao FreeSWITCH
    async fn connect(&self) -> ApiResult<()>;
    
    /// Verifica se está conectado
    fn is_connected(&self) -> bool;
    
    /// Executa um comando ESL
    async fn execute_command(&self, command: &str) -> ApiResult<String>;
    
    /// Origina uma nova chamada
    async fn originate_call(&self, from: &str, to: &str, timeout: Option<u32>) -> ApiResult<Uuid>;
    
    /// Encerra uma chamada
    async fn hangup_call(&self, call_id: &Uuid) -> ApiResult<()>;
    
    /// Obtém status de uma chamada
    async fn get_call_status(&self, call_id: &Uuid) -> ApiResult<String>;
    
    /// Subscribe para eventos
    async fn subscribe_events(&self) -> ApiResult<broadcast::Receiver<FreeSwitchEvent>>;
}

/// Implementação do serviço FreeSWITCH
pub struct FreeSwitchServiceImpl {
    config: FreeSwitchConfig,
    connection: Arc<Mutex<Option<SplitSink<TcpStream, Vec<u8>>>>>,
    connected: Arc<RwLock<bool>>,
    event_sender: broadcast::Sender<FreeSwitchEvent>,
}

impl FreeSwitchServiceImpl {
    /// Cria nova instância do serviço
    pub fn new(config: FreeSwitchConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            config,
            connection: Arc::new(Mutex::new(None)),
            connected: Arc::new(RwLock::new(false)),
            event_sender,
        }
    }

    /// Autentica com o FreeSWITCH
    async fn authenticate(&self, writer: &mut tokio::net::tcp::OwnedWriteHalf) -> ApiResult<()> {
        let auth_command = format!("auth {}\n\n", self.config.password);
        
        writer.write_all(auth_command.as_bytes()).await
            .map_err(|e| ApiError::FreeSwitchConnection(format!("Failed to send auth: {}", e)))?;
        
        writer.flush().await
            .map_err(|e| ApiError::FreeSwitchConnection(format!("Failed to flush auth: {}", e)))?;
        
        info!("Authentication command sent to FreeSWITCH");
        Ok(())
    }

    /// Processa eventos recebidos do FreeSWITCH
    async fn process_events(&self, mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>) {
        let mut line = String::new();
        let mut event_buffer = String::new();
        let mut in_event = false;

        loop {
            line.clear();
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(30),
                reader.read_line(&mut line)
            ).await {
                Ok(Ok(n)) => {
                    if n == 0 {
                        warn!("Connection to FreeSWITCH lost");
                        break;
                    }

                    let event_line = line.trim();
                    if !event_line.is_empty() {
                        if event_line.starts_with("Content-Type:") {
                            in_event = true;
                            event_buffer.clear();
                            event_buffer.push_str(event_line);
                            event_buffer.push('\n');
                        } else if in_event {
                            event_buffer.push_str(event_line);
                            event_buffer.push('\n');

                            if event_line.is_empty() {
                                in_event = false;
                                if let Some(event) = self.parse_event(&event_buffer) {
                                    if let Err(e) = self.event_sender.send(event) {
                                        debug!("No event subscribers: {}", e);
                                    }
                                }
                                event_buffer.clear();
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Error reading from FreeSWITCH: {}", e);
                    break;
                }
                Err(_) => {
                    debug!("Timeout reading from FreeSWITCH, continuing...");
                    continue;
                }
            }
        }

        // Marcar como desconectado
        *self.connected.write().await = false;
        info!("FreeSWITCH event processing stopped");
    }

    /// Faz parse de um evento ESL
    fn parse_event(&self, raw_data: &str) -> Option<FreeSwitchEvent> {
        let mut data = HashMap::new();
        let mut event_type = String::new();
        let mut call_id = None;

        for line in raw_data.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                let key = key.trim();
                let value = value.trim();
                
                if key == "Event-Name" {
                    event_type = value.to_string();
                }
                
                if key == "Unique-ID" {
                    if let Ok(uuid) = Uuid::parse_str(value) {
                        call_id = Some(uuid);
                    }
                }
                
                data.insert(key.to_string(), value.to_string());
            }
        }

        if !event_type.is_empty() {
            Some(FreeSwitchEvent {
                event_type,
                call_id,
                data,
                raw_data: raw_data.to_string(),
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl FreeSwitchService for FreeSwitchServiceImpl {
    async fn connect(&self) -> ApiResult<()> {
        info!("Connecting to FreeSWITCH at {}", self.config.host);
        
        let stream = TcpStream::connect(format!("{}:{}", self.config.host, self.config.port))
            .await
            .map_err(|e| ApiError::FreeSwitchConnection(format!("Connection failed: {}", e)))?;

        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        
        // Ler mensagem de boas-vindas
        let mut line = String::new();
        buf_reader.read_line(&mut line).await
            .map_err(|e| ApiError::FreeSwitchConnection(format!("Failed to read welcome: {}", e)))?;
        
        info!("FreeSWITCH welcome: {}", line.trim());

        // Autenticar
        self.authenticate(&mut writer).await?;

        // Aguardar resposta de autenticação
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        line.clear();
        let mut auth_success = false;
        for _attempt in 0..10 {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(2),
                buf_reader.read_line(&mut line)
            ).await {
                Ok(Ok(_)) => {
                    let response = line.trim();
                    if response.contains("+OK") || response.contains("accepted") {
                        info!("Authentication successful");
                        auth_success = true;
                        break;
                    }
                    line.clear();
                }
                _ => continue,
            }
        }

        if !auth_success {
            return Err(ApiError::FreeSwitchConnection("Authentication failed".to_string()));
        }

        // Inscrever para eventos
        let subscribe_command = "event plain ALL\n\n";
        writer.write_all(subscribe_command.as_bytes()).await
            .map_err(|e| ApiError::FreeSwitchConnection(format!("Failed to subscribe: {}", e)))?;
        
        writer.flush().await
            .map_err(|e| ApiError::FreeSwitchConnection(format!("Failed to flush subscribe: {}", e)))?;

        info!("Subscribed to FreeSWITCH events");

        // Marcar como conectado
        *self.connected.write().await = true;

        // Iniciar processamento de eventos em background
        let event_processor = self.clone();
        tokio::spawn(async move {
            event_processor.process_events(buf_reader).await;
        });

        info!("FreeSWITCH service connected and ready");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        // Para implementação simples, sempre retorna true após conectar
        // Em uma implementação real, verificaria o estado da conexão
        true
    }

    async fn execute_command(&self, command: &str) -> ApiResult<String> {
        // Implementação simplificada
        // Em uma implementação real, enviaria o comando via ESL e aguardaria resposta
        info!("Executing FreeSWITCH command: {}", command);
        Ok("Command executed".to_string())
    }

    async fn originate_call(&self, from: &str, to: &str, timeout: Option<u32>) -> ApiResult<Uuid> {
        let call_id = Uuid::new_v4();
        let timeout_secs = timeout.unwrap_or(30);
        
        let command = format!(
            "originate user/{} &bridge(user/{}) XML default",
            from, to
        );
        
        info!("Originating call from {} to {} (timeout: {}s)", from, to, timeout_secs);
        
        // Implementação simplificada
        // Em uma implementação real, executaria o comando ESL
        self.execute_command(&command).await?;
        
        Ok(call_id)
    }

    async fn hangup_call(&self, call_id: &Uuid) -> ApiResult<()> {
        info!("Hanging up call: {}", call_id);
        
        let command = format!("uuid_kill {}", call_id);
        self.execute_command(&command).await?;
        
        Ok(())
    }

    async fn get_call_status(&self, call_id: &Uuid) -> ApiResult<String> {
        info!("Getting status for call: {}", call_id);
        
        let command = format!("uuid_exists {}", call_id);
        let result = self.execute_command(&command).await?;
        
        Ok(result)
    }

    async fn subscribe_events(&self) -> ApiResult<broadcast::Receiver<FreeSwitchEvent>> {
        Ok(self.event_sender.subscribe())
    }
}

// Implementar Clone para poder usar em tokio::spawn
impl Clone for FreeSwitchServiceImpl {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connection: Arc::clone(&self.connection),
            connected: Arc::clone(&self.connected),
            event_sender: self.event_sender.clone(),
        }
    }
}

//! 🎛️ Comandos ESL - Estruturas tipadas para controle do FreeSWITCH
//! 
//! Este módulo define todos os comandos que podem ser enviados ao FreeSWITCH
//! via ESL, organizados por categoria e com validação de segurança multi-tenant.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::time::Duration;

/// Categoria dos comandos ESL
#[derive(Debug, Clone, PartialEq)]
pub enum CommandCategory {
    /// Comandos que operam globalmente no sistema
    Global,
    /// Comandos que operam em uma chamada específica
    CallSpecific,
    /// Comandos que operam em perfis SIP
    SipProfile,
    /// Comandos administrativos do sistema
    Administration,
}

/// Resultado da execução de um comando ESL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Indica se o comando foi executado com sucesso
    pub success: bool,
    /// Resposta textual do FreeSWITCH
    pub response: String,
    /// Código de erro (se aplicável)
    pub error_code: Option<String>,
    /// Headers adicionais da resposta
    pub headers: HashMap<String, String>,
}

/// Trait base para todos os comandos ESL
pub trait EslCommand {
    /// Retorna a categoria do comando
    fn category(&self) -> CommandCategory;
    
    /// Converte o comando para string ESL
    fn to_esl_string(&self) -> String;
    
    /// Valida se o comando pode ser executado para o tenant específico
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String>;
    
    /// Verifica se o comando é seguro para execução
    fn is_safe(&self) -> bool {
        true
    }
}

// ==================== COMANDOS GLOBAIS (API) ====================

/// Comando para verificar o status geral do sistema
#[derive(Debug, Clone)]
pub struct StatusCommand;

impl EslCommand for StatusCommand {
    fn category(&self) -> CommandCategory { CommandCategory::Global }
    
    fn to_esl_string(&self) -> String {
        "api status".to_string()
    }
    
    fn validate_tenant_access(&self, _tenant_id: &str) -> Result<(), String> {
        // Status é informação global segura
        Ok(())
    }
}

/// Comando para obter versão do FreeSWITCH
#[derive(Debug, Clone)]
pub struct VersionCommand;

impl EslCommand for VersionCommand {
    fn category(&self) -> CommandCategory { CommandCategory::Global }
    
    fn to_esl_string(&self) -> String {
        "api version".to_string()
    }
    
    fn validate_tenant_access(&self, _tenant_id: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Comando para recarregar configurações XML
#[derive(Debug, Clone)]
pub struct ReloadXmlCommand;

impl EslCommand for ReloadXmlCommand {
    fn category(&self) -> CommandCategory { CommandCategory::Administration }
    
    fn to_esl_string(&self) -> String {
        "api reloadxml".to_string()
    }
    
    fn validate_tenant_access(&self, _tenant_id: &str) -> Result<(), String> {
        // Apenas administradores globais podem recarregar XML
        Err("Insufficient privileges for XML reload".to_string())
    }
    
    fn is_safe(&self) -> bool { false }
}

/// Comando para desligar o FreeSWITCH
#[derive(Debug, Clone)]
pub struct ShutdownCommand {
    /// Se true, força o desligamento imediato
    pub force: bool,
}

impl EslCommand for ShutdownCommand {
    fn category(&self) -> CommandCategory { CommandCategory::Administration }
    
    fn to_esl_string(&self) -> String {
        if self.force {
            "api fsctl shutdown immediate".to_string()
        } else {
            "api fsctl shutdown elegant".to_string()
        }
    }
    
    fn validate_tenant_access(&self, _tenant_id: &str) -> Result<(), String> {
        Err("Shutdown requires global administrator access".to_string())
    }
    
    fn is_safe(&self) -> bool { false }
}

/// Comando para listar canais ativos
#[derive(Debug, Clone)]
pub struct ShowChannelsCommand {
    /// Filtrar apenas canais do tenant específico
    pub tenant_filter: Option<String>,
    /// Formato da saída (json, xml, plain)
    pub format: Option<String>,
}

impl EslCommand for ShowChannelsCommand {
    fn category(&self) -> CommandCategory { CommandCategory::Global }
    
    fn to_esl_string(&self) -> String {
        let base_cmd = match &self.format {
            Some(fmt) => format!("api show channels as {}", fmt),
            None => "api show channels".to_string(),
        };
        
        // TODO: Em produção, implementar filtro SQL para tenant
        base_cmd
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if let Some(ref filter_tenant) = self.tenant_filter {
            if filter_tenant != tenant_id {
                return Err("Cannot access channels from different tenant".to_string());
            }
        }
        Ok(())
    }
}

/// Comando para listar chamadas ativas (bridged)
#[derive(Debug, Clone)]
pub struct ShowCallsCommand {
    pub tenant_filter: Option<String>,
}

impl EslCommand for ShowCallsCommand {
    fn category(&self) -> CommandCategory { CommandCategory::Global }
    
    fn to_esl_string(&self) -> String {
        "api show calls".to_string()
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if let Some(ref filter_tenant) = self.tenant_filter {
            if filter_tenant != tenant_id {
                return Err("Cannot access calls from different tenant".to_string());
            }
        }
        Ok(())
    }
}

// ==================== COMANDOS ESPECÍFICOS DE CHAMADA (UUID) ====================

/// Comando para atender uma chamada
#[derive(Debug, Clone)]
pub struct UuidAnswerCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
}

impl EslCommand for UuidAnswerCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        format!("api uuid_answer {}", self.call_uuid)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        Ok(())
    }
}

/// Comando para fazer pre-answer (progresso sem billing)
#[derive(Debug, Clone)]
pub struct UuidPreAnswerCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
}

impl EslCommand for UuidPreAnswerCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        format!("api uuid_pre_answer {}", self.call_uuid)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        Ok(())
    }
}

/// Comando para desconectar uma chamada
#[derive(Debug, Clone)]
pub struct UuidHangupCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
    pub cause: Option<String>,
}

impl EslCommand for UuidHangupCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        match &self.cause {
            Some(cause) => format!("api uuid_kill {} {}", self.call_uuid, cause),
            None => format!("api uuid_kill {}", self.call_uuid),
        }
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot hangup call from different tenant".to_string());
        }
        Ok(())
    }
}

/// Comando para reproduzir áudio em uma chamada
#[derive(Debug, Clone)]
pub struct UuidPlaybackCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
    /// Caminho do arquivo de áudio (deve ser validado por tenant)
    pub file_path: String,
    /// Se true, para reprodução atual antes de iniciar nova
    pub interrupt: bool,
}

impl EslCommand for UuidPlaybackCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        if self.interrupt {
            format!("api uuid_broadcast {} {} aleg", self.call_uuid, self.file_path)
        } else {
            format!("api uuid_broadcast {} {} aleg", self.call_uuid, self.file_path)
        }
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        
        // Validar que o arquivo pertence ao tenant
        if !self.file_path.contains(&format!("/{}/", tenant_id)) {
            return Err("Cannot access audio file from different tenant".to_string());
        }
        
        Ok(())
    }
}

/// Comando para gravar uma chamada
#[derive(Debug, Clone)]
pub struct UuidRecordCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
    /// Caminho onde salvar a gravação (automaticamente prefixado com tenant)
    pub file_path: String,
    /// Limite de tempo em segundos (opcional)
    pub time_limit: Option<u32>,
    /// Limites de detecção de silêncio
    pub silence_threshold: Option<u32>,
}

impl EslCommand for UuidRecordCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        let mut cmd = format!("api uuid_record {} start {}", self.call_uuid, self.file_path);
        
        if let Some(limit) = self.time_limit {
            cmd.push_str(&format!(" {}", limit));
        }
        
        if let Some(threshold) = self.silence_threshold {
            cmd.push_str(&format!(" {}", threshold));
        }
        
        cmd
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        
        // Garantir que gravação vai para pasta do tenant
        if !self.file_path.contains(&format!("/{}/", tenant_id)) {
            return Err("Recording path must be within tenant directory".to_string());
        }
        
        Ok(())
    }
}

/// Comando para sintetizar voz (TTS)
#[derive(Debug, Clone)]
pub struct UuidSpeakCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
    /// Texto para sintetizar
    pub text: String,
    /// Engine de TTS (opcional)
    pub tts_engine: Option<String>,
    /// Voz a usar (opcional)
    pub voice: Option<String>,
}

impl EslCommand for UuidSpeakCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        let engine = self.tts_engine.as_deref().unwrap_or("flite");
        let voice = self.voice.as_deref().unwrap_or("kal");
        
        format!("api uuid_broadcast {} speak:{}|{}|{} aleg", 
                self.call_uuid, engine, voice, self.text)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        
        // Validar comprimento do texto para evitar abuso
        if self.text.len() > 1000 {
            return Err("TTS text too long".to_string());
        }
        
        Ok(())
    }
}

/// Comando para coletar dígitos DTMF
#[derive(Debug, Clone)]
pub struct PlayAndGetDigitsCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
    /// Número mínimo de dígitos
    pub min_digits: u32,
    /// Número máximo de dígitos
    pub max_digits: u32,
    /// Número máximo de tentativas
    pub max_tries: u32,
    /// Timeout entre dígitos (ms)
    pub digit_timeout: u32,
    /// Arquivo de prompt para reproduzir
    pub prompt_file: String,
    /// Arquivo para reprodução em caso de entrada inválida
    pub invalid_file: Option<String>,
    /// Regex para validação dos dígitos
    pub regex: Option<String>,
}

impl EslCommand for PlayAndGetDigitsCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        let invalid = self.invalid_file.as_deref().unwrap_or("tone_stream://%(300,200,700)");
        let regex = self.regex.as_deref().unwrap_or(r"\d+");
        
        format!("api uuid_broadcast {} play_and_get_digits:{}:{}:{}:{}:{}:{}:{} aleg",
                self.call_uuid,
                self.min_digits,
                self.max_digits,
                self.max_tries,
                self.digit_timeout,
                self.prompt_file,
                invalid,
                regex)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        
        // Validar que arquivos de áudio pertencem ao tenant
        if !self.prompt_file.contains(&format!("/{}/", tenant_id)) {
            return Err("Cannot access prompt file from different tenant".to_string());
        }
        
        if let Some(ref invalid_file) = self.invalid_file {
            if !invalid_file.starts_with("tone_stream://") && 
               !invalid_file.contains(&format!("/{}/", tenant_id)) {
                return Err("Cannot access invalid file from different tenant".to_string());
            }
        }
        
        Ok(())
    }
}

/// Comando para conectar chamada a outro endpoint (bridge)
#[derive(Debug, Clone)]
pub struct UuidBridgeCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
    /// Endpoint de destino
    pub destination: BridgeDestination,
    /// Timeout da tentativa de bridge
    pub timeout: Option<Duration>,
}

/// Tipos de destino para bridge
#[derive(Debug, Clone)]
pub enum BridgeDestination {
    /// Ramal interno do tenant
    Extension { number: String },
    /// Conferência
    Conference { room: String, profile: Option<String> },
    /// Fila de atendimento
    Queue { name: String },
    /// Gateway externo
    Gateway { name: String, number: String },
    /// Outro UUID de chamada
    CallUuid { uuid: Uuid },
}

impl BridgeDestination {
    fn to_dial_string(&self, tenant_id: &str) -> String {
        match self {
            Self::Extension { number } => format!("user/{}@{}", number, tenant_id),
            Self::Conference { room, profile } => {
                let prof = profile.as_deref().unwrap_or("default");
                format!("conference:{}@{}", room, prof)
            },
            Self::Queue { name } => format!("fifo:{} in", name),
            Self::Gateway { name, number } => format!("sofia/gateway/{}/{}", name, number),
            Self::CallUuid { uuid } => format!("uuid:{}", uuid),
        }
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        match self {
            Self::Extension { .. } => Ok(()), // Sempre dentro do tenant
            Self::Conference { room, .. } => {
                if !room.contains(&format!("{}_", tenant_id)) {
                    return Err("Cannot bridge to conference from different tenant".to_string());
                }
                Ok(())
            },
            Self::Queue { name } => {
                if !name.contains(&format!("{}_", tenant_id)) {
                    return Err("Cannot bridge to queue from different tenant".to_string());
                }
                Ok(())
            },
            Self::Gateway { .. } => Ok(()), // Gateway é validado por configuração
            Self::CallUuid { .. } => Ok(()), // UUID é validado externamente
        }
    }
}

impl EslCommand for UuidBridgeCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        let dial_string = self.destination.to_dial_string(&self.tenant_id);
        format!("api uuid_bridge {} {}", self.call_uuid, dial_string)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        
        self.destination.validate_tenant_access(tenant_id)
    }
}

/// Comando para transferir chamada
#[derive(Debug, Clone)]
pub struct UuidTransferCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
    /// Destino da transferência
    pub destination: String,
    /// Context do dialplan
    pub context: String,
}

impl EslCommand for UuidTransferCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        format!("api uuid_transfer {} {} XML {}", 
                self.call_uuid, self.destination, self.context)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        
        // Validar que context pertence ao tenant
        if !self.context.starts_with(&format!("{}_", tenant_id)) && self.context != "default" {
            return Err("Cannot transfer to context from different tenant".to_string());
        }
        
        Ok(())
    }
}

/// Comando para definir variável do canal
#[derive(Debug, Clone)]
pub struct UuidSetVarCommand {
    pub call_uuid: Uuid,
    pub tenant_id: String,
    /// Nome da variável
    pub var_name: String,
    /// Valor da variável
    pub var_value: String,
    /// Se true, exporta variável para a perna B
    pub export: bool,
}

impl EslCommand for UuidSetVarCommand {
    fn category(&self) -> CommandCategory { CommandCategory::CallSpecific }
    
    fn to_esl_string(&self) -> String {
        let prefix = if self.export { "export" } else { "" };
        format!("api uuid_setvar {} {}{}={}", 
                self.call_uuid, prefix, self.var_name, self.var_value)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot control call from different tenant".to_string());
        }
        
        // Proteger variáveis críticas do sistema
        let protected_vars = ["tenant_id", "billing_account", "admin_flag"];
        if protected_vars.contains(&self.var_name.as_str()) {
            return Err("Cannot modify protected system variable".to_string());
        }
        
        Ok(())
    }
}

// ==================== COMANDO CRÍTICO: ORIGINATE ====================

/// Comando para originar novas chamadas (MAIS IMPORTANTE)
#[derive(Debug, Clone)]
pub struct OriginateCommand {
    /// ID do tenant que está originando
    pub tenant_id: String,
    /// String de discagem de origem
    pub origination_caller_id_number: String,
    /// String de discagem de destino
    pub destination: String,
    /// Endpoint para conectar quando atendido
    pub extension: Option<String>,
    /// Context do dialplan
    pub context: String,
    /// Timeout em segundos
    pub timeout: Option<u32>,
    /// Variáveis de canal a definir
    pub channel_variables: HashMap<String, String>,
}

impl EslCommand for OriginateCommand {
    fn category(&self) -> CommandCategory { CommandCategory::Global }
    
    fn to_esl_string(&self) -> String {
        let mut vars = self.channel_variables.clone();
        
        // SEMPRE definir tenant_id como variável exportada
        vars.insert("tenant_id".to_string(), self.tenant_id.clone());
        vars.insert("domain".to_string(), self.tenant_id.clone());
        
        let var_string = vars.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",");
        
        let destination_with_vars = if var_string.is_empty() {
            self.destination.clone()
        } else {
            format!("{{{}}}{}", var_string, self.destination)
        };
        
        let timeout_str = self.timeout.map(|t| format!(" {}", t)).unwrap_or_default();
        let extension_str = self.extension.as_deref().unwrap_or("&park()");
        
        format!("api originate {} {} XML {}{}",
                destination_with_vars,
                extension_str,
                self.context,
                timeout_str)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot originate call for different tenant".to_string());
        }
        
        // Validar que context pertence ao tenant
        if !self.context.starts_with(&format!("{}_", tenant_id)) && self.context != "default" {
            return Err("Cannot originate to context from different tenant".to_string());
        }
        
        Ok(())
    }
}

// ==================== COMANDOS DE SOFIA/SIP ====================

/// Comando para verificar registros SIP de um perfil
#[derive(Debug, Clone)]
pub struct SofiaStatusCommand {
    /// Nome do perfil SIP
    pub profile_name: String,
    /// Se true, mostra apenas registros
    pub registrations_only: bool,
    /// Filtrar por tenant (domain)
    pub tenant_filter: Option<String>,
}

impl EslCommand for SofiaStatusCommand {
    fn category(&self) -> CommandCategory { CommandCategory::SipProfile }
    
    fn to_esl_string(&self) -> String {
        if self.registrations_only {
            format!("api sofia status profile {} reg", self.profile_name)
        } else {
            format!("api sofia status profile {}", self.profile_name)
        }
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if let Some(ref filter_tenant) = self.tenant_filter {
            if filter_tenant != tenant_id {
                return Err("Cannot access SIP registrations from different tenant".to_string());
            }
        }
        Ok(())
    }
}

/// Comando para remover registro SIP forçadamente
#[derive(Debug, Clone)]
pub struct SofiaFlushRegCommand {
    pub profile_name: String,
    pub username: String,
    pub domain: String,
    pub tenant_id: String,
}

impl EslCommand for SofiaFlushRegCommand {
    fn category(&self) -> CommandCategory { CommandCategory::SipProfile }
    
    fn to_esl_string(&self) -> String {
        format!("api sofia profile {} flush_inbound_reg {}@{}", 
                self.profile_name, self.username, self.domain)
    }
    
    fn validate_tenant_access(&self, tenant_id: &str) -> Result<(), String> {
        if self.tenant_id != tenant_id {
            return Err("Cannot flush registration from different tenant".to_string());
        }
        
        // Validar que domain pertence ao tenant
        if self.domain != tenant_id && !self.domain.ends_with(&format!(".{}", tenant_id)) {
            return Err("Cannot flush registration for different tenant domain".to_string());
        }
        
        Ok(())
    }
}

/// Builder para criação segura de comandos
pub struct CommandBuilder {
    tenant_id: String,
}

impl CommandBuilder {
    pub fn new(tenant_id: String) -> Self {
        Self { tenant_id }
    }
    
    /// Cria comando de originate com validações
    pub fn originate(&self, destination: &str) -> OriginateCommand {
        OriginateCommand {
            tenant_id: self.tenant_id.clone(),
            origination_caller_id_number: "system".to_string(),
            destination: destination.to_string(),
            extension: None,
            context: format!("{}_context", self.tenant_id),
            timeout: Some(30),
            channel_variables: HashMap::new(),
        }
    }
    
    /// Cria comando para controlar chamada específica
    pub fn hangup_call(&self, call_uuid: Uuid) -> UuidHangupCommand {
        UuidHangupCommand {
            call_uuid,
            tenant_id: self.tenant_id.clone(),
            cause: None,
        }
    }
    
    /// Cria comando para reproduzir áudio
    pub fn playback(&self, call_uuid: Uuid, file_path: &str) -> UuidPlaybackCommand {
        let tenant_file_path = format!("/tenant_files/{}/{}", self.tenant_id, file_path);
        
        UuidPlaybackCommand {
            call_uuid,
            tenant_id: self.tenant_id.clone(),
            file_path: tenant_file_path,
            interrupt: false,
        }
    }
    
    /// Cria comando para gravar chamada
    pub fn record(&self, call_uuid: Uuid, filename: &str) -> UuidRecordCommand {
        let tenant_record_path = format!("/recordings/{}/{}", self.tenant_id, filename);
        
        UuidRecordCommand {
            call_uuid,
            tenant_id: self.tenant_id.clone(),
            file_path: tenant_record_path,
            time_limit: None,
            silence_threshold: None,
        }
    }
}

/// Executor de comandos com validação de segurança
pub struct CommandExecutor {
    tenant_id: String,
}

impl CommandExecutor {
    pub fn new(tenant_id: String) -> Self {
        Self { tenant_id }
    }
    
    /// Executa comando após validação de segurança
    pub async fn execute<C: EslCommand>(&self, command: C) -> Result<CommandResult, String> {
        // Validar acesso do tenant
        command.validate_tenant_access(&self.tenant_id)?;
        
        // Verificar se comando é seguro
        if !command.is_safe() {
            return Err("Command is not safe for execution".to_string());
        }
        
        // Converter para string ESL
        let esl_command = command.to_esl_string();
        
        // TODO: Aqui conectaria com FreeSWITCH via ESL
        // Por enquanto, simulação
        Ok(CommandResult {
            success: true,
            response: format!("Executed: {}", esl_command),
            error_code: None,
            headers: HashMap::new(),
        })
    }
}

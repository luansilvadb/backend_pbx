# 📡 Referência Técnica ESL - FreeSWITCH Multi-Tenant

## Visão Geral da Arquitetura

Este documento serve como referência completa para desenvolvedores que trabalham com o sistema PABX multi-tenant em Rust, utilizando FreeSWITCH como motor de mídia através do Event Socket Layer (ESL).

### Componentes Principais

- **Backend Rust**: Controla toda a lógica de negócio e roteamento
- **FreeSWITCH**: Motor de mídia especializado
- **ESL (Event Socket Layer)**: Interface de comunicação bidirecional
- **Multi-Tenancy**: Isolamento completo entre clientes

## 🎯 Eventos do FreeSWITCH (Notificações para o Backend)

### 1. Ciclo de Vida da Chamada

#### `CHANNEL_CREATE`
**Implementação**: `EslEventType::ChannelCreate`
**Dados**: `EslEventData::ChannelCreate`

```rust
// Exemplo de uso
match event.event_type {
    EslEventType::ChannelCreate => {
        if let EslEventData::ChannelCreate { caller_id_number, destination_number, .. } = &event.data {
            let tenant_id = event.get_tenant_id().unwrap_or("unknown");
            info!("Nova chamada de {} para {} (tenant: {})", 
                  caller_id_number, destination_number, tenant_id);
            
            // Inicializar estruturas de tracking
            call_manager.create_call(event.call_uuid.unwrap(), tenant_id).await?;
        }
    }
}
```

**Headers Críticos**:
- `Unique-ID`: Identificador único da chamada
- `variable_tenant_id`: ID do tenant (configurado previamente)
- `Caller-Caller-ID-Number`: Número de origem
- `Caller-Destination-Number`: Número de destino

**Uso Multi-Tenant**: 
- Primeiro ponto para associar chamada ao tenant
- Extrair `variable_tenant_id` e inicializar contexto
- Aplicar políticas específicas do tenant (billing, gravação, etc.)

#### `CHANNEL_ANSWER`
**Implementação**: `EslEventType::ChannelAnswer`
**Dados**: `EslEventData::ChannelAnswer`

```rust
// Exemplo de tratamento
EslEventType::ChannelAnswer => {
    if let Some(tenant_id) = event.get_tenant_id() {
        // Iniciar billing para o tenant
        billing_service.start_billing(event.call_uuid.unwrap(), tenant_id).await?;
        
        // Ativar gravação se configurada para o tenant
        if tenant_has_recording_policy(tenant_id) {
            freeswitch.start_recording_for_tenant(
                tenant_id, 
                event.call_uuid.unwrap(), 
                &generate_recording_filename(tenant_id)
            ).await?;
        }
    }
}
```

**Uso Multi-Tenant**:
- Início do billing por tenant
- Aplicação de políticas de gravação
- Atualização de métricas de ASR (Answer Seizure Ratio)

#### `CHANNEL_HANGUP`
**Implementação**: `EslEventType::ChannelHangup`
**Dados**: `EslEventData::ChannelHangup`

```rust
// Exemplo de finalização
EslEventType::ChannelHangup => {
    if let EslEventData::ChannelHangup { hangup_cause, duration_seconds, .. } = &event.data {
        let tenant_id = event.get_tenant_id().unwrap();
        
        // Finalizar billing
        let cost = billing_service.finalize_billing(
            event.call_uuid.unwrap(), 
            tenant_id, 
            *duration_seconds
        ).await?;
        
        // Gerar CDR (Call Detail Record)
        cdr_service.create_cdr(event, tenant_id, cost).await?;
        
        // Atualizar estatísticas do tenant
        tenant_stats.update_call_stats(tenant_id, hangup_cause).await?;
    }
}
```

### 2. Interação do Usuário

#### `DTMF`
**Implementação**: `EslEventType::Dtmf`
**Dados**: `EslEventData::Dtmf`

```rust
// Exemplo de navegação IVR
EslEventType::Dtmf => {
    if let EslEventData::Dtmf { digit, .. } = &event.data {
        let tenant_id = event.get_tenant_id().unwrap();
        
        // Processar dígito baseado no IVR do tenant
        let ivr_config = tenant_service.get_ivr_config(tenant_id).await?;
        let next_action = ivr_config.process_digit(*digit, &current_menu_state);
        
        match next_action {
            IvrAction::PlayMessage(file) => {
                freeswitch.playback_for_tenant(
                    tenant_id, 
                    event.call_uuid.unwrap(), 
                    &file
                ).await?;
            }
            IvrAction::Transfer(extension) => {
                let bridge_cmd = CommandBuilder::new(tenant_id.to_string())
                    .build_bridge(event.call_uuid.unwrap(), &extension);
                freeswitch.execute_command_for_tenant(bridge_cmd, tenant_id).await?;
            }
        }
    }
}
```

### 3. Registro e Presença

#### `SOFIA::REGISTER`
**Implementação**: `EslEventType::SofiaRegister`
**Dados**: `EslEventData::SofiaRegistration`

```rust
// Exemplo de registro SIP
EslEventType::SofiaRegister => {
    if let EslEventData::SofiaRegistration { user, realm, contact, .. } = &event.data {
        // Validar que realm pertence ao tenant
        if let Some(tenant_id) = tenant_service.get_tenant_by_domain(realm).await? {
            // Atualizar status no dashboard
            presence_service.update_extension_status(
                &tenant_id, 
                user, 
                ExtensionStatus::Online
            ).await?;
            
            // Notificar supervisores se configurado
            if tenant_has_presence_notifications(&tenant_id) {
                notification_service.notify_extension_online(&tenant_id, user).await?;
            }
        }
    }
}
```

## 🎛️ Comandos para o FreeSWITCH (Ordens do Backend)

### 1. Comandos Globais

#### Status do Sistema
```rust
use crate::services::esl::StatusCommand;

// Verificar status geral
let status_cmd = StatusCommand;
let result = freeswitch.execute_command_for_tenant(status_cmd, "global").await?;
```

#### Listar Chamadas
```rust
use crate::services::esl::ShowCallsCommand;

// Listar chamadas filtradas por tenant
let calls_cmd = ShowCallsCommand {
    tenant_filter: Some(tenant_id.to_string()),
};
let result = freeswitch.execute_command_for_tenant(calls_cmd, tenant_id).await?;
```

### 2. Comandos Específicos de Chamada

#### Originar Chamada (CRÍTICO)
```rust
use crate::services::esl::{OriginateCommand, BridgeDestination};

// Exemplo: Originar chamada para ramal interno
let mut originate_cmd = OriginateCommand {
    tenant_id: tenant_id.to_string(),
    origination_caller_id_number: "1001".to_string(),
    destination: format!("user/1002@{}", tenant_id),
    extension: Some("&park()".to_string()),
    context: format!("{}_context", tenant_id),
    timeout: Some(30),
    channel_variables: {
        let mut vars = HashMap::new();
        vars.insert("tenant_id".to_string(), tenant_id.to_string());
        vars.insert("billing_account".to_string(), tenant_account.to_string());
        vars
    },
};

let result = freeswitch.execute_command_for_tenant(originate_cmd, tenant_id).await?;
```

#### Bridge/Conectar Chamadas
```rust
use crate::services::esl::{UuidBridgeCommand, BridgeDestination};

// Conectar a ramal interno
let bridge_cmd = UuidBridgeCommand {
    call_uuid,
    tenant_id: tenant_id.to_string(),
    destination: BridgeDestination::Extension { 
        number: "1001".to_string() 
    },
    timeout: Some(Duration::from_secs(30)),
};

// Conectar a conferência
let conference_bridge = UuidBridgeCommand {
    call_uuid,
    tenant_id: tenant_id.to_string(),
    destination: BridgeDestination::Conference { 
        room: format!("{}_sales", tenant_id),
        profile: Some("default".to_string())
    },
    timeout: None,
};

// Conectar a fila de atendimento
let queue_bridge = UuidBridgeCommand {
    call_uuid,
    tenant_id: tenant_id.to_string(),
    destination: BridgeDestination::Queue { 
        name: format!("{}_support", tenant_id)
    },
    timeout: None,
};
```

#### Reprodução de Áudio
```rust
use crate::services::esl::UuidPlaybackCommand;

// Reproduzir mensagem personalizada do tenant
let playback_cmd = UuidPlaybackCommand {
    call_uuid,
    tenant_id: tenant_id.to_string(),
    file_path: format!("/tenant_files/{}/welcome.wav", tenant_id),
    interrupt: true,
};

let result = freeswitch.execute_command_for_tenant(playback_cmd, tenant_id).await?;
```

#### Gravação
```rust
use crate::services::esl::UuidRecordCommand;

// Iniciar gravação com path organizado por tenant
let record_cmd = UuidRecordCommand {
    call_uuid,
    tenant_id: tenant_id.to_string(),
    file_path: format!("/recordings/{}/{}/{}.wav", 
                      tenant_id, 
                      Utc::now().format("%Y-%m-%d"), 
                      call_uuid),
    time_limit: Some(3600), // 1 hora máximo
    silence_threshold: Some(30), // 30s de silêncio
};

let result = freeswitch.execute_command_for_tenant(record_cmd, tenant_id).await?;
```

#### Coleta de Dígitos
```rust
use crate::services::esl::PlayAndGetDigitsCommand;

// Coletar código do cliente
let collect_cmd = PlayAndGetDigitsCommand {
    call_uuid,
    tenant_id: tenant_id.to_string(),
    min_digits: 4,
    max_digits: 8,
    max_tries: 3,
    digit_timeout: 5000, // 5 segundos
    prompt_file: format!("/tenant_files/{}/prompts/enter_code.wav", tenant_id),
    invalid_file: Some(format!("/tenant_files/{}/prompts/invalid.wav", tenant_id)),
    regex: Some(r"^\d{4,8}$".to_string()),
};

let result = freeswitch.execute_command_for_tenant(collect_cmd, tenant_id).await?;
```

## 🛡️ Segurança e Isolamento Multi-Tenant

### Validação de Acesso
```rust
use crate::services::esl::TenantValidator;

// Validar acesso a recurso
TenantValidator::validate_resource_access(
    tenant_id, 
    "/recordings/tenant123/call.wav"
)?;

// Validar acesso a chamada
TenantValidator::validate_call_access(
    tenant_id, 
    &call_uuid, 
    &call_headers
)?;

// Validar domínio SIP
TenantValidator::validate_sip_domain(
    tenant_id, 
    "subdomain.tenant123.com"
)?;
```

### Executor Seguro
```rust
use crate::services::esl::CommandExecutor;

// Criar executor com validação automática
let executor = CommandExecutor::new(tenant_id.to_string());

// Todos os comandos são automaticamente validados
let result = executor.execute(any_command).await?;
```

### Builder Seguro
```rust
use crate::services::esl::CommandBuilder;

// Builder garante paths corretos para o tenant
let builder = CommandBuilder::new(tenant_id.to_string());

// Arquivos automaticamente prefixados com tenant
let playback_cmd = builder.playback(call_uuid, "welcome.wav");
// Resulta em: /tenant_files/{tenant_id}/welcome.wav

let record_cmd = builder.record(call_uuid, "recording.wav");
// Resulta em: /recordings/{tenant_id}/recording.wav
```

## 📊 Monitoramento e Métricas

### Métricas por Tenant
```rust
// Coletar métricas específicas do tenant
let tenant_metrics = TenantMetrics {
    active_calls: freeswitch.get_active_calls(tenant_id).await.len(),
    total_calls_today: call_stats.get_daily_count(tenant_id).await,
    average_call_duration: call_stats.get_avg_duration(tenant_id).await,
    answer_seizure_ratio: call_stats.get_asr(tenant_id).await,
    concurrent_call_peak: call_stats.get_peak_concurrent(tenant_id).await,
};
```

### Health Checks
```rust
// Verificar saúde do sistema
let health = HealthCheck {
    freeswitch_connected: freeswitch.get_connection_state().await == EslConnectionState::Connected,
    events_per_second: freeswitch.get_metrics().await.events_per_second,
    command_latency: freeswitch.get_metrics().await.average_command_latency_ms,
    active_tenants: tenant_service.get_active_count().await,
};
```

## 🔄 Padrões de Uso Comuns

### Click-to-Call
```rust
pub async fn click_to_call(
    tenant_id: &str,
    extension: &str,
    external_number: &str
) -> ApiResult<Uuid> {
    // 1. Originar para ramal interno
    let originate_cmd = OriginateCommand {
        tenant_id: tenant_id.to_string(),
        destination: format!("user/{}@{}", extension, tenant_id),
        extension: Some(format!("&bridge(sofia/gateway/{}/{})", 
                               tenant_gateway, external_number)),
        context: format!("{}_context", tenant_id),
        timeout: Some(30),
        channel_variables: {
            let mut vars = HashMap::new();
            vars.insert("tenant_id".to_string(), tenant_id.to_string());
            vars.insert("call_type".to_string(), "click_to_call".to_string());
            vars
        },
    };
    
    let result = freeswitch.execute_command_for_tenant(originate_cmd, tenant_id).await?;
    
    // 2. Retornar UUID da chamada para tracking
    Ok(extract_uuid_from_result(&result.response)?)
}
```

### IVR Dinâmico
```rust
pub async fn handle_ivr_navigation(
    tenant_id: &str,
    call_uuid: Uuid,
    digit: char,
    current_menu: &str
) -> ApiResult<()> {
    let ivr_config = tenant_service.get_ivr_config(tenant_id).await?;
    let menu = ivr_config.get_menu(current_menu)?;
    
    match menu.get_action_for_digit(digit) {
        IvrAction::PlayMessage(file) => {
            freeswitch.playback_for_tenant(tenant_id, call_uuid, &file).await?;
        }
        IvrAction::Transfer(extension) => {
            let bridge_cmd = UuidBridgeCommand {
                call_uuid,
                tenant_id: tenant_id.to_string(),
                destination: BridgeDestination::Extension { number: extension },
                timeout: Some(Duration::from_secs(30)),
            };
            freeswitch.execute_command_for_tenant(bridge_cmd, tenant_id).await?;
        }
        IvrAction::Queue(queue_name) => {
            let queue_cmd = UuidBridgeCommand {
                call_uuid,
                tenant_id: tenant_id.to_string(),
                destination: BridgeDestination::Queue { name: queue_name },
                timeout: None,
            };
            freeswitch.execute_command_for_tenant(queue_cmd, tenant_id).await?;
        }
        IvrAction::Hangup => {
            let hangup_cmd = UuidHangupCommand {
                call_uuid,
                tenant_id: tenant_id.to_string(),
                cause: Some("NORMAL_CLEARING".to_string()),
            };
            freeswitch.execute_command_for_tenant(hangup_cmd, tenant_id).await?;
        }
    }
    
    Ok(())
}
```

### Auto-Dialer
```rust
pub async fn start_campaign(
    tenant_id: &str,
    campaign: &DialerCampaign
) -> ApiResult<()> {
    for contact in &campaign.contacts {
        // Rate limiting por tenant
        if get_concurrent_calls(tenant_id).await >= campaign.max_concurrent {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        
        // Originar chamada
        let originate_cmd = OriginateCommand {
            tenant_id: tenant_id.to_string(),
            origination_caller_id_number: campaign.caller_id.clone(),
            destination: format!("sofia/gateway/{}/{}", 
                               campaign.gateway, contact.phone),
            extension: Some(format!("&lua(campaign_handler.lua {})", campaign.id)),
            context: format!("{}_campaign", tenant_id),
            timeout: Some(campaign.timeout),
            channel_variables: {
                let mut vars = HashMap::new();
                vars.insert("tenant_id".to_string(), tenant_id.to_string());
                vars.insert("campaign_id".to_string(), campaign.id.to_string());
                vars.insert("contact_id".to_string(), contact.id.to_string());
                vars
            },
        };
        
        freeswitch.execute_command_for_tenant(originate_cmd, tenant_id).await?;
        
        // Delay entre chamadas
        tokio::time::sleep(Duration::from_millis(campaign.dial_interval_ms)).await;
    }
    
    Ok(())
}
```

## 🚀 Melhores Práticas

### 1. Gestão de Estado
- Sempre usar UUID para rastrear chamadas
- Manter cache thread-safe de chamadas ativas por tenant
- Implementar cleanup automático de estado obsoleto

### 2. Tratamento de Erros
- Validar tenant antes de qualquer operação
- Implementar retry com backoff exponencial para comandos críticos
- Logar todas as operações com contexto de tenant

### 3. Performance
- Usar pools de conexão para múltiplas instâncias FreeSWITCH
- Implementar cache para configurações de tenant
- Processar eventos de forma assíncrona e não-bloqueante

### 4. Segurança
- Nunca expor UUIDs de chamadas entre tenants diferentes
- Validar todos os paths de arquivo com TenantValidator
- Implementar rate limiting por tenant

Esta documentação fornece a base para implementar um sistema PABX multi-tenant robusto, escalável e seguro usando Rust e FreeSWITCH.

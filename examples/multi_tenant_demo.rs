//! 🚀 Demonstração Multi-Tenant - Exemplo prático do sistema refatorado
//! 
//! Este exemplo demonstra como usar o sistema PABX multi-tenant refatorado,
//! incluindo criação de tenants, originação de chamadas e processamento de eventos.

use backend_pbx::{
    models::{
        CreateTenantRequest, TenantContact, ServicePlan, RecordingPolicy,
        TenantConfigRequest, BillingSettings
    },
    services::{
        TenantServiceImpl, TenantService, FreeSwitchServiceEnhanced, TenantOperations,
        esl::{
            EslEvent, EslEventType, EslEventData, TenantContext,
            CommandBuilder, BridgeDestination, UuidBridgeCommand
        }
    },
    config::AppConfig,
};

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carregar configuração do .env
    let config = AppConfig::load()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Inicializar logging baseado na configuração
    let log_level = match config.logging.level.to_lowercase().as_str() {
        "trace" => "trace",
        "debug" => "debug", 
        "info" => "info",
        "warn" => "warn",
        "error" => "error",
        _ => "info",
    };

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    info!("🚀 Iniciando demonstração do sistema PABX multi-tenant");
    info!("📋 Configuração carregada do .env");

    // 1. Configurar serviços usando configuração do .env
    let tenant_service = Arc::new(TenantServiceImpl::new());
    let freeswitch_service = Arc::new(FreeSwitchServiceEnhanced::new(config.freeswitch.clone()));

    // 2. Criar tenants de demonstração
    info!("📋 Criando tenants de demonstração...");
    create_demo_tenants(&tenant_service).await?;

    // 3. Demonstrar configurações carregadas do .env
    info!("⚙️ Demonstrando configurações do .env...");
    demonstrate_config_usage(&config);

    // 4. Simular eventos ESL e operações de chamada
    info!("📞 Demonstrando operações de chamada multi-tenant...");
    demonstrate_call_operations(&tenant_service, &freeswitch_service).await?;

    // 5. Simular processamento de eventos
    info!("📡 Demonstrando processamento de eventos ESL...");
    demonstrate_event_processing().await?;

    // 6. Demonstrar validações de segurança
    info!("🔒 Demonstrando validações de segurança multi-tenant...");
    demonstrate_security_validations(&tenant_service, &freeswitch_service).await?;

    info!("✅ Demonstração concluída com sucesso!");

    Ok(())
}

/// Cria tenants de demonstração
async fn create_demo_tenants(
    tenant_service: &Arc<TenantServiceImpl>
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Tenant 1: Empresa de Vendas
    let sales_company = CreateTenantRequest {
        id: "sales_corp".to_string(),
        name: "Sales Corporation LTDA".to_string(),
        service_plan: ServicePlan::Professional,
        sip_domain: "sales.pbx.demo".to_string(),
        contact: TenantContact {
            company_name: "Sales Corporation LTDA".to_string(),
            contact_name: "Carlos Vendedor".to_string(),
            email: "carlos@salescorp.com".to_string(),
            phone: "+5511888888888".to_string(),
            address: Some("Av. Vendas, 123".to_string()),
            city: Some("São Paulo".to_string()),
            state: Some("SP".to_string()),
            postal_code: Some("01000-000".to_string()),
            country: "Brasil".to_string(),
        },
        config: Some(TenantConfigRequest {
            default_gateway: Some("sales_gateway".to_string()),
            recording_policy: Some(RecordingPolicy::Always),
            presence_enabled: Some(true),
            conference_settings: None,
            ivr_settings: None,
            timezone: Some("America/Sao_Paulo".to_string()),
            billing_settings: Some(BillingSettings {
                currency: "BRL".to_string(),
                local_rate_per_minute: 0.03,
                national_rate_per_minute: 0.10,
                international_rate_per_minute: 0.35,
                billing_increment: 60,
                grace_period_seconds: 15,
            }),
        }),
        custom_limits: None,
    };

    // Tenant 2: Call Center de Suporte
    let support_center = CreateTenantRequest {
        id: "support_center".to_string(),
        name: "Centro de Suporte 24h".to_string(),
        service_plan: ServicePlan::Enterprise,
        sip_domain: "support.pbx.demo".to_string(),
        contact: TenantContact {
            company_name: "Centro de Suporte 24h".to_string(),
            contact_name: "Ana Atendente".to_string(),
            email: "ana@supportcenter.com".to_string(),
            phone: "+5511777777777".to_string(),
            address: Some("Rua Suporte, 456".to_string()),
            city: Some("Rio de Janeiro".to_string()),
            state: Some("RJ".to_string()),
            postal_code: Some("20000-000".to_string()),
            country: "Brasil".to_string(),
        },
        config: Some(TenantConfigRequest {
            default_gateway: Some("support_gateway".to_string()),
            recording_policy: Some(RecordingPolicy::ExternalOnly),
            presence_enabled: Some(true),
            conference_settings: None,
            ivr_settings: None,
            timezone: Some("America/Sao_Paulo".to_string()),
            billing_settings: None,
        }),
        custom_limits: None,
    };

    // Criar tenants
    let sales_tenant = tenant_service.create_tenant(sales_company).await?;
    let support_tenant = tenant_service.create_tenant(support_center).await?;

    info!("✅ Tenant criado: {} ({})", sales_tenant.name, sales_tenant.id);
    info!("✅ Tenant criado: {} ({})", support_tenant.name, support_tenant.id);

    // Listar tenants criados
    let tenant_list = tenant_service.list_tenants(1, 10).await?;
    info!("📊 Total de tenants: {}", tenant_list.total);

    for tenant_summary in &tenant_list.tenants {
        info!("  - {} ({}): {} chamadas ativas", 
              tenant_summary.name, 
              tenant_summary.id, 
              tenant_summary.active_calls);
    }

    Ok(())
}

/// Demonstra operações de chamada multi-tenant
async fn demonstrate_call_operations(
    tenant_service: &Arc<TenantServiceImpl>,
    freeswitch_service: &Arc<FreeSwitchServiceEnhanced>
) -> Result<(), Box<dyn std::error::Error>> {
    
    // === CENÁRIO 1: Click-to-Call para Sales Corp ===
    info!("🎯 Cenário 1: Click-to-Call para Sales Corp");
    
    let tenant_id = "sales_corp";
    
    // Validar que tenant pode fazer chamadas
    tenant_service.validate_tenant_operation(tenant_id, "originate_call").await?;
    
    // Originar chamada: ramal 1001 para cliente externo
    match freeswitch_service.originate_call_for_tenant(
        tenant_id,
        "1001",  // ramal interno
        "+5511999999999"  // cliente externo
    ).await {
        Ok(call_uuid) => {
            info!("✅ Chamada originada: {}", call_uuid);
            
            // Simular atendimento e reprodução de mensagem
            sleep(Duration::from_secs(2)).await;
            
            // Reproduzir mensagem de boas-vindas do tenant
            if let Err(e) = freeswitch_service.playback_for_tenant(
                tenant_id,
                call_uuid,
                "welcome_sales.wav"
            ).await {
                warn!("Erro ao reproduzir áudio: {}", e);
            }
            
            // Simular encerramento
            sleep(Duration::from_secs(5)).await;
            freeswitch_service.hangup_call_for_tenant(tenant_id, call_uuid).await?;
            info!("📞 Chamada encerrada: {}", call_uuid);
        }
        Err(e) => error!("❌ Erro ao originar chamada: {}", e),
    }

    // === CENÁRIO 2: IVR Inteligente para Support Center ===
    info!("🎯 Cenário 2: IVR Inteligente para Support Center");
    
    let tenant_id = "support_center";
    
    // Simular chamada recebida no support center
    match freeswitch_service.originate_call_for_tenant(
        tenant_id,
        "+5511888888888",  // número externo
        "2000"  // ramal de atendimento
    ).await {
        Ok(call_uuid) => {
            info!("📞 Chamada IVR iniciada: {}", call_uuid);
            
            // Reproduzir menu principal
            freeswitch_service.playback_for_tenant(
                tenant_id,
                call_uuid,
                "ivr_main_menu.wav"
            ).await?;
            
            // Simular seleção "1" para suporte técnico
            sleep(Duration::from_secs(3)).await;
            info!("🔢 Cliente pressionou: 1 (Suporte Técnico)");
            
            // Bridge para fila de suporte técnico
            let builder = CommandBuilder::new(tenant_id.to_string());
            let bridge_cmd = UuidBridgeCommand {
                call_uuid,
                tenant_id: tenant_id.to_string(),
                destination: BridgeDestination::Queue { 
                    name: format!("{}_technical_support", tenant_id)
                },
                timeout: Some(Duration::from_secs(120)),
            };
            
            if let Err(e) = freeswitch_service.execute_command_for_tenant(bridge_cmd, tenant_id).await {
                warn!("Erro ao conectar à fila: {}", e);
            }
            
            info!("🏪 Chamada transferida para fila de suporte técnico");
            
            // Simular atendimento
            sleep(Duration::from_secs(10)).await;
            freeswitch_service.hangup_call_for_tenant(tenant_id, call_uuid).await?;
            info!("📞 Chamada de suporte encerrada: {}", call_uuid);
        }
        Err(e) => error!("❌ Erro no IVR: {}", e),
    }

    // === CENÁRIO 3: Conferência Empresarial ===
    info!("🎯 Cenário 3: Conferência Empresarial");
    
    let tenant_id = "sales_corp";
    
    // Criar 3 chamadas para conferência
    let mut conference_calls = Vec::new();
    
    for participant in ["1001", "1002", "1003"] {
        match freeswitch_service.originate_call_for_tenant(
            tenant_id,
            participant,
            "conference"
        ).await {
            Ok(call_uuid) => {
                info!("👥 Participante {} conectado à conferência: {}", participant, call_uuid);
                conference_calls.push(call_uuid);
                
                // Conectar à sala de conferência do tenant
                let bridge_cmd = UuidBridgeCommand {
                    call_uuid,
                    tenant_id: tenant_id.to_string(),
                    destination: BridgeDestination::Conference { 
                        room: format!("{}_sales_meeting", tenant_id),
                        profile: Some("default".to_string())
                    },
                    timeout: None,
                };
                
                freeswitch_service.execute_command_for_tenant(bridge_cmd, tenant_id).await?;
                sleep(Duration::from_millis(500)).await;
            }
            Err(e) => error!("❌ Erro ao conectar participante {}: {}", participant, e),
        }
    }
    
    info!("🎪 Conferência iniciada com {} participantes", conference_calls.len());
    
    // Simular reunião
    sleep(Duration::from_secs(8)).await;
    
    // Encerrar conferência
    for call_uuid in conference_calls {
        freeswitch_service.hangup_call_for_tenant(tenant_id, call_uuid).await?;
    }
    
    info!("📞 Conferência encerrada");

    Ok(())
}

/// Demonstra processamento de eventos ESL
async fn demonstrate_event_processing() -> Result<(), Box<dyn std::error::Error>> {
    
    // Simular eventos ESL típicos
    let events = vec![
        // Evento de criação de canal
        create_sample_event(
            EslEventType::ChannelCreate,
            "sales_corp",
            EslEventData::ChannelCreate {
                caller_id_number: "1001".to_string(),
                caller_id_name: "João Vendedor".to_string(),
                destination_number: "+5511999999999".to_string(),
                channel_name: "sofia/internal/1001@sales.pbx.demo".to_string(),
                context: "sales_corp_context".to_string(),
            }
        ),
        
        // Evento de atendimento
        create_sample_event(
            EslEventType::ChannelAnswer,
            "sales_corp",
            EslEventData::ChannelAnswer {
                caller_id_number: "1001".to_string(),
                destination_number: "+5511999999999".to_string(),
                answer_time: chrono::Utc::now(),
            }
        ),
        
        // Evento DTMF
        create_sample_event(
            EslEventType::Dtmf,
            "support_center",
            EslEventData::Dtmf {
                digit: '1',
                duration_ms: Some(200),
                source: "inband".to_string(),
            }
        ),
        
        // Evento de registro SIP
        create_sample_event(
            EslEventType::SofiaRegister,
            "support_center",
            EslEventData::SofiaRegistration {
                user: "2001".to_string(),
                realm: "support.pbx.demo".to_string(),
                contact: "sip:2001@192.168.1.100:5060".to_string(),
                expires: Some(3600),
                user_agent: Some("X-Lite/5.0".to_string()),
            }
        ),
    ];

    // Processar eventos simulados
    for event in events {
        process_sample_event(&event).await;
        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Demonstra validações de segurança multi-tenant
async fn demonstrate_security_validations(
    tenant_service: &Arc<TenantServiceImpl>,
    freeswitch_service: &Arc<FreeSwitchServiceEnhanced>
) -> Result<(), Box<dyn std::error::Error>> {
    
    info!("🔍 Testando validações de segurança...");
    
    // === TESTE 1: Tentativa de acesso cruzado entre tenants ===
    info!("Test 1: Acesso cruzado entre tenants");
    
    let sales_call_uuid = Uuid::new_v4();
    tenant_service.add_active_call("sales_corp", sales_call_uuid).await;
    
    // Tentar encerrar chamada do sales_corp usando support_center (deve falhar)
    match freeswitch_service.hangup_call_for_tenant("support_center", sales_call_uuid).await {
        Ok(_) => error!("❌ FALHA DE SEGURANÇA: Tenant conseguiu acessar chamada de outro tenant!"),
        Err(_) => info!("✅ Isolamento funcionando: Acesso negado corretamente"),
    }
    
    // === TESTE 2: Validação de limites de recursos ===
    info!("Test 2: Limites de recursos por tenant");
    
    // Obter limites do tenant
    let limits = tenant_service.get_tenant_limits("sales_corp").await?;
    info!("📊 Limites do sales_corp: {} chamadas simultâneas", limits.max_concurrent_calls);
    
    // === TESTE 3: Validação de paths de arquivo ===
    info!("Test 3: Validação de paths de arquivo");
    
    let test_call_uuid = Uuid::new_v4();
    tenant_service.add_active_call("sales_corp", test_call_uuid).await;
    
    // Tentar reproduzir arquivo de outro tenant (deve falhar)
    match freeswitch_service.playback_for_tenant(
        "sales_corp",
        test_call_uuid,
        "/tenant_files/support_center/private_message.wav"  // arquivo de outro tenant
    ).await {
        Ok(_) => error!("❌ FALHA DE SEGURANÇA: Acesso a arquivo de outro tenant permitido!"),
        Err(_) => info!("✅ Isolamento de arquivos funcionando: Acesso negado corretamente"),
    }
    
    // Reproduzir arquivo válido do próprio tenant (deve funcionar)
    match freeswitch_service.playback_for_tenant(
        "sales_corp",
        test_call_uuid,
        "welcome.wav"  // será automaticamente prefixado com /tenant_files/sales_corp/
    ).await {
        Ok(_) => info!("✅ Acesso a arquivo próprio: Permitido corretamente"),
        Err(e) => warn!("Erro ao acessar arquivo próprio: {}", e),
    }
    
    // === TESTE 4: Rate Limiting ===
    info!("Test 4: Rate Limiting por tenant");
    
    // Simular múltiplas tentativas de originação rápida
    for i in 1..=5 {
        match tenant_service.validate_tenant_operation("sales_corp", "originate_call").await {
            Ok(_) => info!("✅ Chamada {} autorizada", i),
            Err(e) => {
                if e.to_string().contains("Rate limit") {
                    info!("🚦 Rate limit ativado na chamada {}: {}", i, e);
                    break;
                } else {
                    warn!("Erro inesperado: {}", e);
                }
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    info!("✅ Todos os testes de segurança concluídos");

    Ok(())
}

// === FUNÇÕES AUXILIARES ===

/// Cria evento ESL de exemplo
fn create_sample_event(
    event_type: EslEventType,
    tenant_id: &str,
    data: EslEventData,
) -> EslEvent {
    use std::collections::HashMap;
    
    EslEvent {
        event_type,
        call_uuid: Some(Uuid::new_v4()),
        timestamp: chrono::Utc::now(),
        tenant_context: TenantContext {
            tenant_id: Some(tenant_id.to_string()),
            domain: Some(format!("{}.pbx.demo", tenant_id)),
            context: Some(format!("{}_context", tenant_id)),
            gateway: Some(format!("{}_gateway", tenant_id)),
        },
        data,
        raw_headers: HashMap::new(),
    }
}

/// Processa evento ESL de exemplo
async fn process_sample_event(event: &EslEvent) {
    let tenant_id = event.get_tenant_id().unwrap_or("unknown");
    
    match &event.event_type {
        EslEventType::ChannelCreate => {
            if let EslEventData::ChannelCreate { caller_id_number, destination_number, .. } = &event.data {
                info!("📞 [{}] Nova chamada: {} → {}", 
                      tenant_id, caller_id_number, destination_number);
            }
        }
        
        EslEventType::ChannelAnswer => {
            if let EslEventData::ChannelAnswer { caller_id_number, destination_number, .. } = &event.data {
                info!("✅ [{}] Chamada atendida: {} → {}", 
                      tenant_id, caller_id_number, destination_number);
            }
        }
        
        EslEventType::Dtmf => {
            if let EslEventData::Dtmf { digit, .. } = &event.data {
                info!("🔢 [{}] DTMF recebido: {}", tenant_id, digit);
            }
        }
        
        EslEventType::SofiaRegister => {
            if let EslEventData::SofiaRegistration { user, realm, .. } = &event.data {
                info!("📝 [{}] Registro SIP: {}@{}", tenant_id, user, realm);
            }
        }
        
        _ => {
            info!("📡 [{}] Evento: {:?}", tenant_id, event.event_type);
        }
    }
}

/// Demonstra como as configurações são carregadas do .env
fn demonstrate_config_usage(config: &AppConfig) {
    info!("📊 Configurações carregadas do arquivo .env:");
    
    info!("🌐 Servidor:");
    info!("  • Host: {}", config.server.host);
    info!("  • Porta: {}", config.server.port);
    info!("  • CORS habilitado: {}", config.server.cors_enabled);
    info!("  • Origens CORS: {:?}", config.server.cors_origins);
    
    info!("📡 FreeSWITCH:");
    info!("  • Host: {}", config.freeswitch.host);
    info!("  • Porta: {}", config.freeswitch.port);
    info!("  • Timeout: {}s", config.freeswitch.timeout_seconds);
    info!("  • Máx. tentativas: {}", config.freeswitch.max_retries);
    
    info!("🗄️ Banco de Dados:");
    info!("  • URL: {}", config.database.url);
    info!("  • Máx. conexões: {}", config.database.max_connections);
    info!("  • Timeout: {}s", config.database.timeout_seconds);
    
    info!("🔐 Segurança:");
    info!("  • JWT expiração: {}h", config.security.jwt_expiration_hours);
    if config.security.jwt_secret.contains("dev") {
        info!("  • ⚠️ JWT secret de desenvolvimento detectado!");
    }
    if config.security.encryption_key.contains("dev") {
        info!("  • ⚠️ Chave de criptografia de desenvolvimento detectada!");
    }
    
    info!("🏢 Multi-Tenant:");
    info!("  • Diretório de arquivos: {}", config.multi_tenant.tenant_files_base_path);
    info!("  • Diretório de gravações: {}", config.multi_tenant.recordings_base_path);
    info!("  • Máx. chamadas simultâneas padrão: {}", config.multi_tenant.default_max_concurrent_calls);
    info!("  • Máx. ramais padrão: {}", config.multi_tenant.default_max_extensions);
    
    info!("📊 Monitoramento:");
    info!("  • Métricas habilitadas: {}", config.monitoring.metrics_enabled);
    if config.monitoring.metrics_enabled {
        info!("  • Porta de métricas: {}", config.monitoring.metrics_port);
    }
    info!("  • Health check habilitado: {}", config.monitoring.health_check_enabled);
    
    info!("🔄 Cache:");
    match &config.cache.redis_url {
        Some(url) => info!("  • Redis: {}", url),
        None => info!("  • Redis: não configurado"),
    }
    info!("  • TTL padrão: {}s", config.cache.ttl_seconds);
    
    info!("📁 Arquivos:");
    info!("  • Diretório de upload: {}", config.files.upload_dir);
    info!("  • Tamanho máx. upload: {} bytes", config.files.max_upload_size);
    info!("  • Tipos permitidos: {:?}", config.files.allowed_file_types);
    
    info!("🚀 Aplicação:");
    info!("  • Nome: {}", config.app.name);
    info!("  • Versão: {}", config.app.version);
    info!("  • Ambiente: {}", config.app.environment);
    info!("  • Debug: {}", config.app.debug);
    
    if config.is_development() {
        info!("⚠️ Rodando em modo de desenvolvimento");
        info!("💡 Para produção, ajuste as seguintes variáveis no .env:");
        info!("  • ENVIRONMENT=production");
        info!("  • DEBUG=false");
        info!("  • JWT_SECRET=<chave-forte-32-caracteres>");
        info!("  • ENCRYPTION_KEY=<chave-forte-32-caracteres>");
        info!("  • DATABASE_URL=postgresql://...");
    }
}

//! 🚀 Backend PBX - Main Application Entry Point
//! 
//! Ponto de entrada principal da aplicação que carrega configurações do .env,
//! inicializa serviços e inicia o servidor HTTP com todas as funcionalidades.

use backend_pbx::{
    api::{create_router, AppState},
    config::AppConfig,
    services::{CallManager, FreeSwitchServiceImpl, SipUserServiceImpl, TenantServiceImpl},
};
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Carregar configuração primeiro (inclui dotenv)
    let config = AppConfig::load()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    // Inicializar logging com base na configuração
    initialize_logging(&config)?;

    info!("🚀 Starting Backend PBX - Multi-Tenant VoIP Platform");
    info!("📋 Configuration loaded from .env and environment variables");

    // Validar e criar diretórios necessários
    create_required_directories(&config).await?;

    // Inicializar serviços
    info!("🔧 Initializing services...");
    
    let freeswitch_service = Arc::new(FreeSwitchServiceImpl::new(config.freeswitch.clone()));
    let tenant_service = Arc::new(TenantServiceImpl::new());
    let sip_user_service = Arc::new(SipUserServiceImpl::new());
    let call_manager = Arc::new(CallManager::new(freeswitch_service));

    // Inicializar tenants de demonstração se em desenvolvimento
    if config.is_development() {
        info!("🏢 Initializing demo tenants for development...");
        if let Err(e) = tenant_service.initialize_demo_tenants().await {
            error!("Failed to initialize demo tenants: {}", e);
        }

        info!("📞 Initializing demo SIP users for development...");
        if let Err(e) = sip_user_service.initialize_demo_users().await {
            error!("Failed to initialize demo SIP users: {}", e);
        }
    }

    // Inicializar Call Manager (conecta ao FreeSWITCH)
    info!("📡 Connecting to FreeSWITCH at {}", config.freeswitch_address());
    if let Err(e) = call_manager.start().await {
        error!("❌ Failed to start Call Manager: {}", e);
        if config.is_production() {
            return Err(anyhow::anyhow!("Call Manager initialization failed: {}", e));
        } else {
            info!("⚠️  Continuing in development mode without FreeSWITCH connection");
        }
    } else {
        info!("✅ Call Manager initialized successfully");
    }

    // Criar estado da aplicação
    let app_state = AppState {
        config: config.clone(),
        call_manager,
        tenant_service,
        sip_user_service,
    };

    // Criar router da API
    let app = create_router(app_state);

    // Configurar listener
    let listener = tokio::net::TcpListener::bind(&config.server_address())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", config.server_address(), e))?;

    // Log de informações do servidor
    log_server_info(&config);

    // Configurar graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 Shutdown signal received");
    };

    // Iniciar servidor HTTP com graceful shutdown
    info!("🎯 Backend PBX is ready to serve requests!");
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    info!("👋 Backend PBX shut down gracefully");
    Ok(())
}

/// Inicializa o sistema de logging baseado na configuração
fn initialize_logging(config: &AppConfig) -> anyhow::Result<()> {
    let level = match config.logging.level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let env_filter = EnvFilter::from_default_env()
        .add_directive(format!("backend_pbx={}", level).parse()?)
        .add_directive("tower_http=debug".parse()?)
        .add_directive("axum=info".parse()?);

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match config.logging.format.to_lowercase().as_str() {
        "json" => {
            subscriber
                .with(fmt::layer().json())
                .init();
        }
        _ => {
            subscriber
                .with(fmt::layer().pretty())
                .init();
        }
    }

    Ok(())
}

/// Cria diretórios necessários se não existirem
async fn create_required_directories(config: &AppConfig) -> anyhow::Result<()> {
    use tokio::fs;

    let directories = [
        &config.multi_tenant.tenant_files_base_path,
        &config.multi_tenant.recordings_base_path,
        &config.files.upload_dir,
    ];

    for dir in directories {
        if let Err(e) = fs::create_dir_all(dir).await {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                error!("Failed to create directory {}: {}", dir, e);
                return Err(anyhow::anyhow!("Directory creation failed: {}", e));
            }
        } else {
            info!("📁 Created directory: {}", dir);
        }
    }

    Ok(())
}

/// Log das informações do servidor e endpoints disponíveis
fn log_server_info(config: &AppConfig) {
    let base_url = format!("http://{}", config.server_address());
    
    info!("🌐 HTTP server listening on {}", config.server_address());
    info!("🚀 Code-First API Documentation (gerada automaticamente):");
    info!("  📚 Swagger UI: {}/swagger-ui", base_url);
    info!("  📄 OpenAPI JSON: {}/api-docs/openapi.json", base_url);
    info!("  📊 Doc Info: {}/api-docs/info", base_url);
    
    info!("🔗 Available endpoints:");
    info!("  📊 System");
    info!("    • GET  /health                     - Health check");
    info!("    • GET  /api/v1/status              - System status");
    info!("    • GET  /metrics                    - Prometheus metrics");
    
    info!("  🏢 Tenants");
    info!("    • GET  /api/tenants                - List tenants");
    info!("    • POST /api/tenants                - Create tenant");
    info!("    • GET  /api/tenants/{{id}}           - Get tenant details");
    info!("    • PUT  /api/tenants/{{id}}           - Update tenant");
    info!("    • GET  /api/tenants/{{id}}/usage     - Tenant usage statistics");
    info!("    • GET  /api/tenants/{{id}}/health    - Tenant health status");
    
    info!("  📞 Calls");
    info!("    • GET  /api/v1/calls               - List active calls");
    info!("    • POST /api/v1/calls               - Originate new call");
    info!("    • GET  /api/v1/calls/{{id}}          - Get call details");
    info!("    • POST /api/v1/calls/{{id}}/hangup   - Hangup call");

    // Log de configurações importantes
    info!("⚙️  Configuration summary:");
    info!("  🌍 Environment: {}", config.app.environment);
    info!("  🔍 Log level: {}", config.logging.level);
    info!("  📡 FreeSWITCH: {}", config.freeswitch_address());
    info!("  🗄️  Database: {}", config.database.url.split('@').last().unwrap_or("***"));
    
    if config.monitoring.metrics_enabled {
        info!("  📊 Metrics: enabled on port {}", config.monitoring.metrics_port);
    }
    
    if config.cache.redis_url.is_some() {
        info!("  🔄 Cache: Redis enabled");
    }
    
    if config.server.cors_enabled {
        info!("  🌐 CORS: enabled for {}", config.server.cors_origins.join(", "));
    }

    // Informações sobre Code-First Documentation
    info!("📖 Code-First Documentation Status:");
    info!("  ✨ Documentação: 100% gerada automaticamente do código");
    info!("  🔄 Sincronização: Automática (impossível divergir)");
    info!("  📝 Manutenção: Zero (sem arquivos .yaml/.json para manter)");
    info!("  🔒 Type Safety: Garantido em tempo de compilação");
    info!("  🚀 Abordagem: Single Source of Truth (código)");

    // Warnings para desenvolvimento
    if config.is_development() {
        info!("⚠️  Development mode warnings:");
        
        if config.security.jwt_secret.contains("dev") {
            info!("  🔐 Using development JWT secret - change for production!");
        }
        
        if config.security.encryption_key.contains("dev") {
            info!("  🔐 Using development encryption key - change for production!");
        }
        
        if config.app.debug {
            info!("  🐛 Debug mode enabled");
        }
    }
}
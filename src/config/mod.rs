//! ⚙️ Configuration - Gestão de configurações da aplicação com .env
//! 
//! Este módulo gerencia todas as configurações da aplicação, carregando
//! valores de arquivos .env e variáveis de ambiente com fallbacks sensatos.

use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use tracing::{info, warn};

/// Erros de configuração
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] env::VarError),
    
    #[error("Parse error for variable {var}: {source}")]
    Parse {
        var: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuração completa da aplicação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Configurações do servidor HTTP
    pub server: ServerConfig,
    /// Configurações do FreeSWITCH ESL
    pub freeswitch: FreeSwitchConfig,
    /// Configurações de banco de dados
    pub database: DatabaseConfig,
    /// Configurações de logging
    pub logging: LoggingConfig,
    /// Configurações de segurança
    pub security: SecurityConfig,
    /// Configurações de monitoramento
    pub monitoring: MonitoringConfig,
    /// Configurações multi-tenant
    pub multi_tenant: MultiTenantConfig,
    /// Configurações de notificações
    pub notifications: NotificationConfig,
    /// Configurações de cache
    pub cache: CacheConfig,
    /// Configurações de arquivos
    pub files: FileConfig,
    /// Configurações avançadas
    pub advanced: AdvancedConfig,
    /// Informações da aplicação
    pub app: AppInfo,
}

/// Configuração do servidor HTTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub cors_origins: Vec<String>,
    pub compression_enabled: bool,
}

/// Configuração do FreeSWITCH ESL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeSwitchConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub timeout_seconds: u32,
    pub max_retries: u32,
    pub retry_interval_seconds: u32,
}

/// Configuração do banco de dados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
    pub sql_logging: bool,
}

/// Configuração de logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

/// Configuração de segurança
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u32,
    pub encryption_key: String,
}

/// Configuração de monitoramento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub health_check_enabled: bool,
}

/// Configuração multi-tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTenantConfig {
    pub tenant_files_base_path: String,
    pub recordings_base_path: String,
    pub default_max_concurrent_calls: u32,
    pub default_max_extensions: u32,
    pub default_max_monthly_minutes: u32,
}

/// Configuração de notificações
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub smtp: SmtpConfig,
    pub webhook_url: Option<String>,
}

/// Configuração SMTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub from: String,
}

/// Configuração de cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: Option<String>,
    pub ttl_seconds: u32,
}

/// Configuração de arquivos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub upload_dir: String,
    pub max_upload_size: u64,
    pub allowed_file_types: Vec<String>,
}

/// Configurações avançadas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub worker_pool_size: u32,
    pub event_buffer_size: usize,
    pub heartbeat_interval_seconds: u32,
}

/// Informações da aplicação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub debug: bool,
}

impl AppConfig {
    /// Carrega configuração a partir do .env e variáveis de ambiente
    pub fn load() -> Result<Self, ConfigError> {
        // Carregar .env se existir
        if let Err(e) = dotenv::dotenv() {
            warn!("Failed to load .env file: {}", e);
            info!("Continuing with environment variables only");
        } else {
            info!("Successfully loaded .env file");
        }

        let config = Self {
            server: ServerConfig::load()?,
            freeswitch: FreeSwitchConfig::load()?,
            database: DatabaseConfig::load()?,
            logging: LoggingConfig::load()?,
            security: SecurityConfig::load()?,
            monitoring: MonitoringConfig::load()?,
            multi_tenant: MultiTenantConfig::load()?,
            notifications: NotificationConfig::load()?,
            cache: CacheConfig::load()?,
            files: FileConfig::load()?,
            advanced: AdvancedConfig::load()?,
            app: AppInfo::load()?,
        };

        config.validate()?;
        config.log_configuration();

        Ok(config)
    }

    /// Valida a configuração
    fn validate(&self) -> Result<(), ConfigError> {
        // Validar porta do servidor
        if self.server.port == 0 {
            return Err(ConfigError::Validation("Server port cannot be 0".to_string()));
        }

        // Validar configurações do FreeSWITCH
        if self.freeswitch.port == 0 {
            return Err(ConfigError::Validation("FreeSWITCH port cannot be 0".to_string()));
        }

        if self.freeswitch.password.is_empty() {
            return Err(ConfigError::Validation("FreeSWITCH password cannot be empty".to_string()));
        }

        // Validar JWT secret em produção
        if self.is_production() && self.security.jwt_secret.len() < 32 {
            return Err(ConfigError::Validation(
                "JWT secret must be at least 32 characters in production".to_string()
            ));
        }

        // Validar encryption key
        if self.security.encryption_key.len() != 32 {
            return Err(ConfigError::Validation(
                "Encryption key must be exactly 32 characters".to_string()
            ));
        }

        Ok(())
    }

    /// Log da configuração carregada
    fn log_configuration(&self) {
        info!("🚀 Configuration loaded successfully");
        info!("📡 Server: {}:{}", self.server.host, self.server.port);
        info!("📞 FreeSWITCH: {}:{}", self.freeswitch.host, self.freeswitch.port);
        info!("🗄️  Database: {}", self.database.url.split('@').last().unwrap_or("hidden"));
        info!("🌍 Environment: {}", self.app.environment);
        info!("🔍 Log level: {}", self.logging.level);
        info!("🏢 Multi-tenant: {} tenants max", self.multi_tenant.default_max_concurrent_calls);
        
        if self.monitoring.metrics_enabled {
            info!("📊 Metrics enabled on port {}", self.monitoring.metrics_port);
        }
        
        if self.cache.redis_url.is_some() {
            info!("🔄 Redis cache enabled");
        }
    }

    /// Retorna se está em ambiente de desenvolvimento
    pub fn is_development(&self) -> bool {
        self.app.environment.to_lowercase() == "development"
    }

    /// Retorna se está em ambiente de produção
    pub fn is_production(&self) -> bool {
        self.app.environment.to_lowercase() == "production"
    }

    /// Retorna endereço completo do servidor
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Retorna endereço completo do FreeSWITCH
    pub fn freeswitch_address(&self) -> String {
        format!("{}:{}", self.freeswitch.host, self.freeswitch.port)
    }
}

// ========== IMPLEMENTAÇÕES DE CARREGAMENTO ==========

impl ServerConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            host: get_env_or_default("SERVER_HOST", "127.0.0.1"),
            port: parse_env_or_default("SERVER_PORT", 3000)?,
            cors_enabled: parse_env_or_default("CORS_ENABLED", true)?,
            cors_origins: get_env_or_default("CORS_ORIGINS", "http://localhost:3000")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            compression_enabled: parse_env_or_default("COMPRESSION_ENABLED", true)?,
        })
    }
}

impl FreeSwitchConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            host: get_env_or_default("FREESWITCH_HOST", "127.0.0.1"),
            port: parse_env_or_default("FREESWITCH_PORT", 8021)?,
            password: get_env_or_default("FREESWITCH_PASSWORD", "ClueCon"),
            timeout_seconds: parse_env_or_default("FREESWITCH_TIMEOUT", 30)?,
            max_retries: parse_env_or_default("FREESWITCH_MAX_RETRIES", 5)?,
            retry_interval_seconds: parse_env_or_default("FREESWITCH_RETRY_INTERVAL", 5)?,
        })
    }
}

impl DatabaseConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            url: get_env_or_default("DATABASE_URL", "sqlite://./backend_pbx.db"),
            max_connections: parse_env_or_default("DATABASE_MAX_CONNECTIONS", 10)?,
            timeout_seconds: parse_env_or_default("DATABASE_TIMEOUT", 30)?,
            sql_logging: parse_env_or_default("SQL_LOGGING", false)?,
        })
    }
}

impl LoggingConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            level: get_env_or_default("LOG_LEVEL", "info"),
            format: get_env_or_default("LOG_FORMAT", "pretty"),
        })
    }
}

impl SecurityConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            jwt_secret: get_env_or_default("JWT_SECRET", "dev-jwt-secret-key-change-in-production"),
            jwt_expiration_hours: parse_env_or_default("JWT_EXPIRATION", 24)?,
            encryption_key: get_env_or_default("ENCRYPTION_KEY", "dev-encryption-key-32-chars-long"),
        })
    }
}

impl MonitoringConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            metrics_enabled: parse_env_or_default("METRICS_ENABLED", true)?,
            metrics_port: parse_env_or_default("METRICS_PORT", 9090)?,
            health_check_enabled: parse_env_or_default("HEALTH_CHECK_ENABLED", true)?,
        })
    }
}

impl MultiTenantConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            tenant_files_base_path: get_env_or_default("TENANT_FILES_BASE_PATH", "./tenant_files"),
            recordings_base_path: get_env_or_default("RECORDINGS_BASE_PATH", "./recordings"),
            default_max_concurrent_calls: parse_env_or_default("DEFAULT_MAX_CONCURRENT_CALLS", 10)?,
            default_max_extensions: parse_env_or_default("DEFAULT_MAX_EXTENSIONS", 50)?,
            default_max_monthly_minutes: parse_env_or_default("DEFAULT_MAX_MONTHLY_MINUTES", 1000)?,
        })
    }
}

impl NotificationConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            smtp: SmtpConfig::load()?,
            webhook_url: env::var("WEBHOOK_URL").ok(),
        })
    }
}

impl SmtpConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            host: get_env_or_default("SMTP_HOST", "smtp.gmail.com"),
            port: parse_env_or_default("SMTP_PORT", 587)?,
            user: get_env_or_default("SMTP_USER", "dev@example.com"),
            password: get_env_or_default("SMTP_PASSWORD", "dev-password"),
            from: get_env_or_default("SMTP_FROM", "noreply@example.com"),
        })
    }
}

impl CacheConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            redis_url: env::var("REDIS_URL").ok(),
            ttl_seconds: parse_env_or_default("CACHE_TTL", 3600)?,
        })
    }
}

impl FileConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            upload_dir: get_env_or_default("UPLOAD_DIR", "./uploads"),
            max_upload_size: parse_env_or_default("MAX_UPLOAD_SIZE", 10485760)?,
            allowed_file_types: get_env_or_default("ALLOWED_FILE_TYPES", "wav,mp3,mp4,aac")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }
}

impl AdvancedConfig {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            worker_pool_size: parse_env_or_default("WORKER_POOL_SIZE", 4)?,
            event_buffer_size: parse_env_or_default("EVENT_BUFFER_SIZE", 10000)?,
            heartbeat_interval_seconds: parse_env_or_default("HEARTBEAT_INTERVAL", 60)?,
        })
    }
}

impl AppInfo {
    fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            name: get_env_or_default("APP_NAME", "Backend PBX"),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: get_env_or_default("ENVIRONMENT", "development"),
            debug: parse_env_or_default("DEBUG", true)?,
        })
    }
}

// ========== FUNÇÕES AUXILIARES ==========

/// Obtém variável de ambiente ou usa valor padrão
fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Faz parse de variável de ambiente ou usa valor padrão
fn parse_env_or_default<T>(key: &str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr + Clone,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match env::var(key) {
        Ok(val) => val.parse().map_err(|e| ConfigError::Parse {
            var: key.to_string(),
            source: Box::new(e),
        }),
        Err(_) => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config_loading() {
        // Limpar variáveis de ambiente para teste
        for key in [
            "SERVER_HOST", "SERVER_PORT", "FREESWITCH_HOST", "FREESWITCH_PORT"
        ] {
            env::remove_var(key);
        }

        let config = AppConfig::load().expect("Should load default config");
        
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.freeswitch.host, "127.0.0.1");
        assert_eq!(config.freeswitch.port, 8021);
        assert_eq!(config.app.environment, "development");
    }

    #[test]
    fn test_env_override() {
        env::set_var("SERVER_PORT", "8080");
        env::set_var("FREESWITCH_HOST", "192.168.1.100");
        
        let config = AppConfig::load().expect("Should load config with env overrides");
        
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.freeswitch.host, "192.168.1.100");
        
        // Limpar após teste
        env::remove_var("SERVER_PORT");
        env::remove_var("FREESWITCH_HOST");
    }

    #[test]
    fn test_config_validation() {
        env::set_var("SERVER_PORT", "0");
        
        let result = AppConfig::load();
        assert!(result.is_err());
        
        env::remove_var("SERVER_PORT");
    }
}
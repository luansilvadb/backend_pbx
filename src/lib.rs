//! 🚀 Backend PBX - Plataforma VoIP Multi-Tenant
//! 
//! Backend modular em Rust para controle de servidor FreeSWITCH via ESL (Event Socket Library).
//! 
//! ## Arquitetura
//! 
//! - **API REST**: Axum + Swagger/OpenAPI para interface HTTP
//! - **Service Layer**: Abstração da comunicação com FreeSWITCH
//! - **Models**: Estruturas de dados tipadas e validadas
//! - **Configuration**: Configuração externa via YAML + env vars
//! - **Observability**: Logging estruturado e metrics

pub mod api;
pub mod config;
pub mod models;
pub mod services;

pub use config::AppConfig;
pub use models::{ApiError, ApiResult};

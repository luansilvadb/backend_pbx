//! 📡 Event Socket Layer (ESL) - Módulos para comunicação com FreeSWITCH
//! 
//! Este módulo contém todas as estruturas e enums necessários para comunicação
//! robusta com FreeSWITCH via ESL, incluindo suporte multi-tenant.

pub mod events;
pub mod commands;
pub mod types;

pub use events::*;
pub use commands::*;
pub use types::*;

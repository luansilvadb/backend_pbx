//! 🔀 FreeSWITCH Integration Handlers
//! 
//! Handlers para integração direta com FreeSWITCH via mod_xml_curl
//! e outros mecanismos de integração.

use crate::api::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use tracing::{info, warn, instrument};

/// Query parameters enviados pelo FreeSWITCH para directory lookups
#[derive(Debug, Deserialize)]
pub struct DirectoryQuery {
    /// Domínio solicitado
    pub domain: Option<String>,
    /// Usuário solicitado
    pub user: Option<String>,
    /// Chave de busca
    pub key: Option<String>,
    /// Valor de busca
    pub value: Option<String>,
    /// Seção solicitada
    pub section: Option<String>,
    /// Tag solicitada
    pub tag: Option<String>,
    /// Key_name para auth
    pub key_name: Option<String>,
    /// Key_value para auth
    pub key_value: Option<String>,
}

/// Endpoint principal para directory lookup do FreeSWITCH
/// Este endpoint é chamado pelo mod_xml_curl para resolver usuários SIP
#[instrument(skip(app_state))]
pub async fn freeswitch_directory(
    State(app_state): State<AppState>,
    Query(params): Query<DirectoryQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    info!("FreeSWITCH directory request: {:?}", params);
    
    // Determinar domínio (tenant_id)
    let domain = match params.domain.as_deref().or(params.key_value.as_deref()) {
        Some(d) => {
            // Mapear domínio para tenant_id
            match d {
                "127.0.0.1" => "demo_company",
                "localhost" => "demo_company",
                _ => d
            }
        },
        None => {
            warn!("No domain provided in directory request");
            return Ok(Html(empty_xml_response()));
        }
    };
    
    // Determinar usuário (extension)
    let user = match params.user.as_deref().or(params.value.as_deref()) {
        Some(u) => u,
        None => {
            warn!("No user provided in directory request");
            return Ok(Html(empty_xml_response()));
        }
    };
    
    // Buscar usuário no backend
    match app_state.sip_user_service.get_user_by_extension(domain, user).await {
        Ok(Some(sip_user)) => {
            info!("Found SIP user {} in domain {}", user, domain);
            
            let xml = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<document type="freeswitch/xml">
  <section name="directory">
    <domain name="127.0.0.1">
      <params>
        <param name="dial-string" value="{{presence_id=${{dialed_user}}@${{dialed_domain}}}}${{sofia_contact(${{dialed_user}}@${{dialed_domain}})}}"/>
      </params>
      <groups>
        <group name="default">
          <users>
            <user id="{}" number-alias="{}">
              <params>
                <param name="password" value="{}"/>
                <param name="vm-password" value="{}"/>
                <param name="vm-email-all-messages" value="true"/>
              </params>
              <variables>
                <variable name="tenant_id" value="{}"/>
                <variable name="user_context" value="default"/>
                <variable name="effective_caller_id_name" value="{}"/>
                <variable name="effective_caller_id_number" value="{}"/>
                <variable name="outbound_caller_id_name" value="{}"/>
                <variable name="outbound_caller_id_number" value="{}"/>
                <variable name="callgroup" value="{}"/>
                <variable name="pickupgroup" value="{}"/>
              </variables>
            </user>
          </users>
        </group>
      </groups>
    </domain>
  </section>
</document>"#,
                user,                            // user id
                user,                            // number-alias
                "demo123",                       // password (senha demo para desenvolvimento)
                user,                            // vm-password
                sip_user.tenant_id,             // tenant_id variable
                sip_user.display_name,          // effective_caller_id_name
                sip_user.extension,             // effective_caller_id_number
                sip_user.display_name,          // outbound_caller_id_name
                sip_user.extension,             // outbound_caller_id_number
                sip_user.tenant_id,             // callgroup
                sip_user.tenant_id              // pickupgroup
            );
            
            Ok(Html(xml))
        }
        Ok(None) => {
            warn!("SIP user {} not found in domain {}", user, domain);
            Ok(Html(empty_xml_response()))
        }
        Err(e) => {
            warn!("Error looking up SIP user {}: {}", user, e);
            Ok(Html(empty_xml_response()))
        }
    }
}

/// Endpoint para autenticação específica (quando FreeSWITCH faz auth check)
#[instrument(skip(app_state))]
pub async fn freeswitch_auth(
    State(app_state): State<AppState>,
    Query(params): Query<DirectoryQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    info!("FreeSWITCH auth request: {:?}", params);
    
    // Determinar domínio e usuário
    let domain = match params.domain.as_deref().or(params.key_value.as_deref()) {
        Some(d) => {
            // Mapear domínio para tenant_id
            match d {
                "127.0.0.1" => "demo_company",
                "localhost" => "demo_company",
                _ => d
            }
        },
        None => {
            warn!("No domain provided in auth request");
            return Ok(Html(empty_xml_response()));
        }
    };
    
    let user = match params.user.as_deref().or(params.value.as_deref()) {
        Some(u) => u,
        None => {
            warn!("No user provided in auth request");
            return Ok(Html(empty_xml_response()));
        }
    };
    
    // Buscar usuário e verificar se existe
    match app_state.sip_user_service.get_user_by_extension(domain, user).await {
        Ok(Some(sip_user)) => {
            info!("Auth check for SIP user {} in domain {}", user, domain);
            
            // Retornar XML com senha demo para desenvolvimento
            let xml = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<document type="freeswitch/xml">
  <section name="directory">
    <domain name="127.0.0.1">
      <params>
        <param name="dial-string" value="{{presence_id=${{dialed_user}}@${{dialed_domain}}}}${{sofia_contact(${{dialed_user}}@${{dialed_domain}})}}"/>
      </params>
      <groups>
        <group name="default">
          <users>
            <user id="{}" number-alias="{}">
              <params>
                <param name="password" value="{}"/>
                <param name="vm-password" value="{}"/>
                <param name="vm-email-all-messages" value="true"/>
              </params>
              <variables>
                <variable name="tenant_id" value="{}"/>
                <variable name="user_context" value="default"/>
                <variable name="effective_caller_id_name" value="{}"/>
                <variable name="effective_caller_id_number" value="{}"/>
                <variable name="outbound_caller_id_name" value="{}"/>
                <variable name="outbound_caller_id_number" value="{}"/>
                <variable name="callgroup" value="{}"/>
                <variable name="pickupgroup" value="{}"/>
              </variables>
            </user>
          </users>
        </group>
      </groups>
    </domain>
  </section>
</document>"#,
                user,                            // user id
                user,                            // number-alias
                "demo123",                       // password (senha demo)
                user,                            // vm-password
                sip_user.tenant_id,             // tenant_id variable
                sip_user.display_name,          // effective_caller_id_name
                sip_user.extension,             // effective_caller_id_number
                sip_user.display_name,          // outbound_caller_id_name
                sip_user.extension,             // outbound_caller_id_number
                sip_user.tenant_id,             // callgroup
                sip_user.tenant_id              // pickupgroup
            );
            
            Ok(Html(xml))
        }
        Ok(None) => {
            warn!("Auth failed - SIP user {} not found in domain {}", user, domain);
            Ok(Html(empty_xml_response()))
        }
        Err(e) => {
            warn!("Auth error looking up SIP user {}: {}", user, e);
            Ok(Html(empty_xml_response()))
        }
    }
}

/// Retorna documento XML vazio para casos onde usuário não é encontrado
fn empty_xml_response() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<document type="freeswitch/xml">
</document>"#.to_string()
}

/// Formatar senha para XML (mascarada por segurança)
#[allow(dead_code)]
fn format_password_for_xml(_password_hash: &str) -> String {
    // Por segurança, não retornar o hash real no XML
    // FreeSWITCH vai usar a autenticação via callback se necessário
    "callback_auth".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_xml_response() {
        let xml = empty_xml_response();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("document type=\"freeswitch/xml\""));
    }

    #[test]
    fn test_format_password_for_xml() {
        let result = format_password_for_xml("some_hash");
        assert_eq!(result, "callback_auth");
    }
}

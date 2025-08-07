//! 📞 SIP User Service - Gerenciamento de usuários e extensões SIP
//! 
//! Este serviço gerencia usuários SIP, suas credenciais, registros e
//! integração com o FreeSWITCH para autenticação de softphones.

use crate::models::{
    ApiError, ApiResult,
    SipUser, SipUserStatus, SipUserResponse, SipUserListResponse,
    CreateSipUserRequest, UpdateSipUserRequest,
    SipRegistrationInfo, SipUserUsageStats,
};

use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};
use uuid::Uuid;
use chrono::Utc;

/// Trait principal para operações de usuários SIP
#[async_trait]
pub trait SipUserService: Send + Sync {
    /// Cria novo usuário SIP
    async fn create_user(&self, tenant_id: &str, request: CreateSipUserRequest) -> ApiResult<SipUserResponse>;
    
    /// Lista usuários SIP de um tenant
    async fn list_users(&self, tenant_id: &str, page: u32, per_page: u32) -> ApiResult<SipUserListResponse>;
    
    /// Obtém usuário por ID
    async fn get_user(&self, tenant_id: &str, user_id: Uuid) -> ApiResult<Option<SipUserResponse>>;
    
    /// Obtém usuário por extensão
    async fn get_user_by_extension(&self, tenant_id: &str, extension: &str) -> ApiResult<Option<SipUserResponse>>;
    
    /// Atualiza usuário SIP
    async fn update_user(&self, tenant_id: &str, user_id: Uuid, request: UpdateSipUserRequest) -> ApiResult<SipUserResponse>;
    
    /// Remove usuário SIP
    async fn delete_user(&self, tenant_id: &str, user_id: Uuid) -> ApiResult<()>;
    
    /// Autentica usuário SIP (para FreeSWITCH)
    async fn authenticate_user(&self, tenant_id: &str, extension: &str, password: &str) -> ApiResult<bool>;
    
    /// Atualiza registro SIP
    async fn update_registration(&self, tenant_id: &str, extension: &str, ip: Option<String>, user_agent: Option<String>) -> ApiResult<()>;
    
    /// Lista registros ativos
    async fn list_registrations(&self, tenant_id: &str) -> ApiResult<Vec<SipRegistrationInfo>>;
    
    /// Obtém estatísticas de uso
    async fn get_user_usage(&self, tenant_id: &str, user_id: Uuid) -> ApiResult<SipUserUsageStats>;
    
    /// Valida se extensão está disponível
    async fn is_extension_available(&self, tenant_id: &str, extension: &str) -> ApiResult<bool>;
    
    /// Reseta senha do usuário
    async fn reset_password(&self, tenant_id: &str, user_id: Uuid, new_password: &str) -> ApiResult<()>;
    
    /// Bloqueia/desbloqueia usuário
    async fn set_user_status(&self, tenant_id: &str, user_id: Uuid, status: SipUserStatus) -> ApiResult<()>;
}

/// Implementação em memória do serviço de usuários SIP
pub struct SipUserServiceImpl {
    /// Usuários organizados por tenant -> extension -> user
    users: Arc<RwLock<HashMap<String, HashMap<String, SipUser>>>>,
    /// Índice por ID para busca rápida: tenant -> user_id -> extension
    user_index: Arc<RwLock<HashMap<String, HashMap<Uuid, String>>>>,
    /// Registros ativos: tenant -> extension -> registration_info
    active_registrations: Arc<RwLock<HashMap<String, HashMap<String, SipRegistrationInfo>>>>,
}

impl SipUserServiceImpl {
    /// Cria nova instância do serviço
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            user_index: Arc::new(RwLock::new(HashMap::new())),
            active_registrations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gera hash da senha
    fn hash_password(password: &str) -> ApiResult<String> {
        hash(password, DEFAULT_COST)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to hash password: {}", e)))
    }

    /// Verifica senha
    fn verify_password(password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }

    /// Valida dados do usuário
    fn validate_user_request(request: &CreateSipUserRequest) -> ApiResult<()> {
        // Validar extensão
        if request.extension.is_empty() || request.extension.len() > 20 {
            return Err(ApiError::BadRequest("Extension must be 1-20 characters".to_string()));
        }

        if !request.extension.chars().all(|c| c.is_ascii_digit()) {
            return Err(ApiError::BadRequest("Extension must contain only digits".to_string()));
        }

        // Validar nome
        if request.display_name.is_empty() || request.display_name.len() > 100 {
            return Err(ApiError::BadRequest("Display name must be 1-100 characters".to_string()));
        }

        // Validar senha
        if request.password.len() < 6 {
            return Err(ApiError::BadRequest("Password must be at least 6 characters".to_string()));
        }

        // Validar email se fornecido
        if let Some(email) = &request.email {
            if !email.contains('@') || email.len() > 255 {
                return Err(ApiError::BadRequest("Invalid email format".to_string()));
            }
        }

        Ok(())
    }

    /// Inicializa usuários demo para development
    #[instrument(skip(self))]
    pub async fn initialize_demo_users(&self) -> ApiResult<()> {
        info!("Initializing demo SIP users...");

        // Usuários para demo_company
        let demo_users = vec![
            CreateSipUserRequest {
                extension: "1001".to_string(),
                display_name: "João Silva".to_string(),
                email: Some("joao@democompany.com".to_string()),
                password: "demo123".to_string(),
                config: None,
                limits: None,
            },
            CreateSipUserRequest {
                extension: "1002".to_string(),
                display_name: "Maria Santos".to_string(),
                email: Some("maria@democompany.com".to_string()),
                password: "demo123".to_string(),
                config: None,
                limits: None,
            },
        ];

        for user_req in demo_users {
            if let Err(e) = self.create_user("demo_company", user_req).await {
                warn!("Failed to create demo user: {}", e);
            }
        }

        // Usuários para acme_corp
        let acme_users = vec![
            CreateSipUserRequest {
                extension: "2001".to_string(),
                display_name: "Alice Johnson".to_string(),
                email: Some("alice@acme.corp".to_string()),
                password: "acme123".to_string(),
                config: None,
                limits: None,
            },
            CreateSipUserRequest {
                extension: "2002".to_string(),
                display_name: "Bob Wilson".to_string(),
                email: Some("bob@acme.corp".to_string()),
                password: "acme123".to_string(),
                config: None,
                limits: None,
            },
        ];

        for user_req in acme_users {
            if let Err(e) = self.create_user("acme_corp", user_req).await {
                warn!("Failed to create acme user: {}", e);
            }
        }

        info!("Demo SIP users initialized successfully");
        Ok(())
    }
}

#[async_trait]
impl SipUserService for SipUserServiceImpl {
    #[instrument(skip(self, request))]
    async fn create_user(&self, tenant_id: &str, request: CreateSipUserRequest) -> ApiResult<SipUserResponse> {
        // Validar request
        Self::validate_user_request(&request)?;

        // Verificar se extensão já existe
        if !self.is_extension_available(tenant_id, &request.extension).await? {
            return Err(ApiError::BadRequest("Extension already exists".to_string()));
        }

        // Hash da senha
        let password_hash = Self::hash_password(&request.password)?;

        // Criar usuário
        let user = SipUser::new(tenant_id.to_string(), request, password_hash);
        let user_id = user.id;
        let extension = user.extension.clone();
        let response = user.to_response(false);

        // Armazenar usuário
        {
            let mut users = self.users.write().await;
            let tenant_users = users.entry(tenant_id.to_string()).or_insert_with(HashMap::new);
            tenant_users.insert(extension.clone(), user);
        }

        // Atualizar índice
        {
            let mut user_index = self.user_index.write().await;
            let tenant_index = user_index.entry(tenant_id.to_string()).or_insert_with(HashMap::new);
            tenant_index.insert(user_id, extension.clone());
        }

        info!("Created SIP user {} ({}) for tenant {}", extension, response.display_name, tenant_id);
        Ok(response)
    }

    #[instrument(skip(self))]
    async fn list_users(&self, tenant_id: &str, page: u32, per_page: u32) -> ApiResult<SipUserListResponse> {
        let page = page.max(1);
        let per_page = per_page.min(100).max(1);

        let users = self.users.read().await;
        let empty_map = HashMap::new();
        let tenant_users = users.get(tenant_id).unwrap_or(&empty_map);

        let total = tenant_users.len() as u64;
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

        // Converter para vec e ordenar por extensão
        let mut user_list: Vec<_> = tenant_users.values().collect();
        user_list.sort_by(|a, b| a.extension.cmp(&b.extension));

        // Aplicar paginação
        let skip = ((page - 1) * per_page) as usize;
        let users_page: Vec<SipUserResponse> = user_list
            .into_iter()
            .skip(skip)
            .take(per_page as usize)
            .map(|u| u.to_response(false)) // TODO: verificar se está registrado
            .collect();

        Ok(SipUserListResponse {
            users: users_page,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    #[instrument(skip(self))]
    async fn get_user(&self, tenant_id: &str, user_id: Uuid) -> ApiResult<Option<SipUserResponse>> {
        // Buscar extensão pelo ID
        let user_index = self.user_index.read().await;
        let tenant_index = user_index.get(tenant_id);
        
        if let Some(extension) = tenant_index.and_then(|idx| idx.get(&user_id)) {
            // Buscar usuário pela extensão
            let users = self.users.read().await;
            if let Some(user) = users.get(tenant_id).and_then(|tu| tu.get(extension)) {
                return Ok(Some(user.to_response(false))); // TODO: verificar registro
            }
        }

        Ok(None)
    }

    #[instrument(skip(self))]
    async fn get_user_by_extension(&self, tenant_id: &str, extension: &str) -> ApiResult<Option<SipUserResponse>> {
        let users = self.users.read().await;
        
        if let Some(user) = users.get(tenant_id).and_then(|tu| tu.get(extension)) {
            Ok(Some(user.to_response(false))) // TODO: verificar registro
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, request))]
    async fn update_user(&self, tenant_id: &str, user_id: Uuid, request: UpdateSipUserRequest) -> ApiResult<SipUserResponse> {
        // Buscar usuário
        let user_index = self.user_index.read().await;
        let extension = user_index
            .get(tenant_id)
            .and_then(|idx| idx.get(&user_id))
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?
            .clone();
        drop(user_index);

        // Atualizar usuário
        let mut users = self.users.write().await;
        let user = users
            .get_mut(tenant_id)
            .and_then(|tu| tu.get_mut(&extension))
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        // Aplicar atualizações
        if let Some(display_name) = request.display_name {
            user.display_name = display_name;
        }

        if let Some(email) = request.email {
            user.email = Some(email);
        }

        if let Some(password) = request.password {
            user.password_hash = Self::hash_password(&password)?;
        }

        if let Some(status) = request.status {
            user.status = status;
        }

        if let Some(config) = request.config {
            user.config = config;
        }

        if let Some(limits) = request.limits {
            user.limits = limits;
        }

        user.updated_at = Utc::now();

        info!("Updated SIP user {} for tenant {}", extension, tenant_id);
        Ok(user.to_response(false))
    }

    #[instrument(skip(self))]
    async fn delete_user(&self, tenant_id: &str, user_id: Uuid) -> ApiResult<()> {
        // Buscar extensão
        let mut user_index = self.user_index.write().await;
        let extension = user_index
            .get_mut(tenant_id)
            .and_then(|idx| idx.remove(&user_id))
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        // Remover usuário
        let mut users = self.users.write().await;
        if let Some(tenant_users) = users.get_mut(tenant_id) {
            tenant_users.remove(&extension);
        }

        // Remover registro ativo se existir
        let mut registrations = self.active_registrations.write().await;
        if let Some(tenant_regs) = registrations.get_mut(tenant_id) {
            tenant_regs.remove(&extension);
        }

        info!("Deleted SIP user {} for tenant {}", extension, tenant_id);
        Ok(())
    }

    #[instrument(skip(self, password))]
    async fn authenticate_user(&self, tenant_id: &str, extension: &str, password: &str) -> ApiResult<bool> {
        let users = self.users.read().await;
        
        if let Some(user) = users.get(tenant_id).and_then(|tu| tu.get(extension)) {
            // Verificar se usuário está ativo
            if user.status != SipUserStatus::Active {
                warn!("Authentication failed for {} - user not active", extension);
                return Ok(false);
            }

            // Verificar senha
            let is_valid = Self::verify_password(password, &user.password_hash);
            
            if is_valid {
                info!("Authentication successful for extension {} in tenant {}", extension, tenant_id);
            } else {
                warn!("Authentication failed for extension {} - invalid password", extension);
            }

            Ok(is_valid)
        } else {
            warn!("Authentication failed for extension {} - user not found", extension);
            Ok(false)
        }
    }

    #[instrument(skip(self))]
    async fn update_registration(&self, tenant_id: &str, extension: &str, ip: Option<String>, user_agent: Option<String>) -> ApiResult<()> {
        // Atualizar informações do usuário
        {
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(tenant_id).and_then(|tu| tu.get_mut(extension)) {
                user.update_last_registration(ip.clone(), user_agent.clone());
            }
        }

        // Atualizar registro ativo
        {
            let mut registrations = self.active_registrations.write().await;
            let tenant_regs = registrations.entry(tenant_id.to_string()).or_insert_with(HashMap::new);
            
            if let Some(user) = self.get_user_by_extension(tenant_id, extension).await? {
                let reg_info = SipRegistrationInfo {
                    user_id: user.id,
                    extension: extension.to_string(),
                    is_registered: true,
                    contact: ip.clone().map(|ip| format!("sip:{}@{}:5060", extension, ip)),
                    expires: Some(3600), // 1 hora
                    user_agent,
                    ip_address: ip,
                    registered_at: Some(Utc::now()),
                };

                tenant_regs.insert(extension.to_string(), reg_info);
            }
        }

        info!("Updated registration for extension {} in tenant {}", extension, tenant_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_registrations(&self, tenant_id: &str) -> ApiResult<Vec<SipRegistrationInfo>> {
        let registrations = self.active_registrations.read().await;
        
        Ok(registrations
            .get(tenant_id)
            .map(|regs| regs.values().cloned().collect::<Vec<_>>())
            .unwrap_or_else(Vec::new))
    }

    #[instrument(skip(self))]
    async fn get_user_usage(&self, tenant_id: &str, user_id: Uuid) -> ApiResult<SipUserUsageStats> {
        // TODO: Implementar cálculo real de estatísticas
        // Por enquanto, retorna dados simulados
        
        if let Some(user) = self.get_user(tenant_id, user_id).await? {
            Ok(SipUserUsageStats {
                user_id,
                extension: user.extension,
                total_calls: 0,
                answered_calls: 0,
                missed_calls: 0,
                total_minutes: 0,
                peak_concurrent_calls: 0,
                period_start: Utc::now() - chrono::Duration::days(30),
                period_end: Utc::now(),
            })
        } else {
            Err(ApiError::NotFound("User not found".to_string()))
        }
    }

    #[instrument(skip(self))]
    async fn is_extension_available(&self, tenant_id: &str, extension: &str) -> ApiResult<bool> {
        let users = self.users.read().await;
        Ok(!users.get(tenant_id).map_or(false, |tu| tu.contains_key(extension)))
    }

    #[instrument(skip(self, new_password))]
    async fn reset_password(&self, tenant_id: &str, user_id: Uuid, new_password: &str) -> ApiResult<()> {
        if new_password.len() < 6 {
            return Err(ApiError::BadRequest("Password must be at least 6 characters".to_string()));
        }

        let request = UpdateSipUserRequest {
            display_name: None,
            email: None,
            password: Some(new_password.to_string()),
            status: None,
            config: None,
            limits: None,
        };

        self.update_user(tenant_id, user_id, request).await?;
        info!("Password reset for user {} in tenant {}", user_id, tenant_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn set_user_status(&self, tenant_id: &str, user_id: Uuid, status: SipUserStatus) -> ApiResult<()> {
        let request = UpdateSipUserRequest {
            display_name: None,
            email: None,
            password: None,
            status: Some(status.clone()),
            config: None,
            limits: None,
        };

        self.update_user(tenant_id, user_id, request).await?;
        info!("Status changed to {:?} for user {} in tenant {}", status, user_id, tenant_id);
        Ok(())
    }
}

impl Default for SipUserServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

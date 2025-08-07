# ⚙️ Configuração com .env - Backend PBX

## 🎯 Visão Geral

O Backend PBX agora utiliza arquivos `.env` para gerenciar todas as configurações da aplicação, proporcionando uma forma mais organizada, segura e flexível de configurar o sistema para diferentes ambientes.

## 📁 Estrutura de Arquivos

```
backend_pbx/
├── .env.example          # Template com todas as variáveis disponíveis
├── .env                  # Configurações locais (não commitado)
├── .gitignore           # Protege arquivos sensíveis
└── src/config/mod.rs    # Módulo de configuração refatorado
```

## 🚀 Como Usar

### 1. **Configuração Inicial**

```bash
# Copiar o template
cp .env.example .env

# Editar as configurações conforme necessário
nano .env  # ou seu editor preferido
```

### 2. **Executar a Aplicação**

```bash
# A aplicação carrega automaticamente o .env
cargo run

# Ou executar o exemplo de demonstração
cargo run --example multi_tenant_demo
```

### 3. **Override com Variáveis de Ambiente**

```bash
# Sobrescrever configurações específicas
SERVER_PORT=8080 LOG_LEVEL=debug cargo run
```

## 📊 Categorias de Configuração

### 🌐 **Servidor HTTP**

```env
# Endereço e porta do servidor
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# CORS
CORS_ENABLED=true
CORS_ORIGINS=http://localhost:3000,http://localhost:8080

# Compressão
COMPRESSION_ENABLED=true
```

### 📡 **FreeSWITCH ESL**

```env
# Conexão com FreeSWITCH
FREESWITCH_HOST=127.0.0.1
FREESWITCH_PORT=8021
FREESWITCH_PASSWORD=ClueCon

# Configurações de conexão
FREESWITCH_TIMEOUT=30
FREESWITCH_MAX_RETRIES=5
FREESWITCH_RETRY_INTERVAL=5
```

### 🗄️ **Banco de Dados**

```env
# URL de conexão
DATABASE_URL=postgresql://user:password@localhost/backend_pbx
# Para SQLite: sqlite://./backend_pbx.db

# Pool de conexões
DATABASE_MAX_CONNECTIONS=10
DATABASE_TIMEOUT=30

# Logging SQL (desenvolvimento)
SQL_LOGGING=false
```

### 🔐 **Segurança**

```env
# JWT (JSON Web Tokens)
JWT_SECRET=your-super-secret-jwt-key-32-chars
JWT_EXPIRATION=24

# Criptografia
ENCRYPTION_KEY=your-encryption-key-32-chars-long
```

⚠️ **IMPORTANTE**: Em produção, use chaves fortes e únicas!

### 🏢 **Multi-Tenant**

```env
# Diretórios organizados por tenant
TENANT_FILES_BASE_PATH=./tenant_files
RECORDINGS_BASE_PATH=./recordings

# Limites padrão por tenant
DEFAULT_MAX_CONCURRENT_CALLS=10
DEFAULT_MAX_EXTENSIONS=50
DEFAULT_MAX_MONTHLY_MINUTES=1000
```

### 📊 **Monitoramento**

```env
# Métricas Prometheus
METRICS_ENABLED=true
METRICS_PORT=9090

# Health checks
HEALTH_CHECK_ENABLED=true
```

### 🔄 **Cache Redis**

```env
# Redis (opcional)
REDIS_URL=redis://127.0.0.1:6379
CACHE_TTL=3600
```

### 📁 **Arquivos**

```env
# Upload de arquivos
UPLOAD_DIR=./uploads
MAX_UPLOAD_SIZE=10485760
ALLOWED_FILE_TYPES=wav,mp3,mp4,aac
```

### 📧 **Notificações**

```env
# SMTP
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=noreply@yourcompany.com

# Webhooks
WEBHOOK_URL=https://your-webhook-endpoint.com/notifications
```

## 🌍 Configuração por Ambiente

### **Desenvolvimento** (`.env`)

```env
ENVIRONMENT=development
DEBUG=true
LOG_LEVEL=debug
LOG_FORMAT=pretty

# Configurações relaxadas para dev
JWT_SECRET=dev-jwt-secret-key-change-in-production
ENCRYPTION_KEY=dev-encryption-key-32-chars-long
DATABASE_URL=sqlite://./backend_pbx.db
```

### **Produção** (`.env.production`)

```env
ENVIRONMENT=production
DEBUG=false
LOG_LEVEL=info
LOG_FORMAT=json

# Configurações seguras para produção
JWT_SECRET=super-secure-jwt-key-32-characters
ENCRYPTION_KEY=super-secure-encryption-32-chars
DATABASE_URL=postgresql://user:strong_password@db-server/backend_pbx

# Limites mais rigorosos
FREESWITCH_TIMEOUT=10
DATABASE_TIMEOUT=15
```

### **Staging** (`.env.staging`)

```env
ENVIRONMENT=staging
DEBUG=false
LOG_LEVEL=debug
LOG_FORMAT=json

# Configurações intermediárias
DATABASE_URL=postgresql://user:password@staging-db/backend_pbx
```

## 🔧 Funcionalidades Avançadas

### **Validação Automática**

O sistema valida automaticamente as configurações na inicialização:

```rust
// Validações implementadas:
- Portas não podem ser 0
- JWT secret deve ter ≥32 caracteres em produção
- Encryption key deve ter exatamente 32 caracteres
- URLs de banco devem ser válidas
```

### **Logs Informativos**

```
🚀 Configuration loaded successfully
📡 Server: 127.0.0.1:3000
📞 FreeSWITCH: 127.0.0.1:8021
🗄️  Database: localhost/backend_pbx
🌍 Environment: development
🔍 Log level: debug
📊 Metrics enabled on port 9090
🔄 Redis cache enabled
```

### **Detecção de Problemas**

```
⚠️ Development mode warnings:
🔐 Using development JWT secret - change for production!
🔐 Using development encryption key - change for production!
🐛 Debug mode enabled
```

## 🛡️ Segurança

### **Proteção de Arquivos**

O `.gitignore` está configurado para proteger:

```gitignore
# Arquivos .env contêm credenciais - NUNCA commitar!
.env
.env.local
.env.production
.env.staging

# Manter apenas .env.example para referência
!.env.example
```

### **Boas Práticas**

1. **Nunca commitar** arquivos `.env` reais
2. **Usar chaves fortes** em produção (≥32 caracteres)
3. **Rotacionar credenciais** regularmente
4. **Limitar acesso** aos arquivos de configuração
5. **Usar secrets do CI/CD** para deploy

## 🔄 Migração de Configuração

### **Do YAML para .env**

```yaml
# config.yaml (antigo)
server:
  host: "127.0.0.1"
  port: 3000
freeswitch:
  host: "127.0.0.1"
  port: 8021
```

```env
# .env (novo)
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
FREESWITCH_HOST=127.0.0.1
FREESWITCH_PORT=8021
```

### **Vantagens da Migração**

✅ **Organização**: Todas as configurações em um local  
✅ **Segurança**: Proteção automática contra commits acidentais  
✅ **Flexibilidade**: Override fácil com variáveis de ambiente  
✅ **Padrão**: Segue convenções da indústria (12-factor app)  
✅ **Validação**: Verificação automática de valores  
✅ **Documentação**: Template completo com explicações  

## 📚 Exemplos de Uso

### **Desenvolvimento Local**

```bash
# Usar configurações padrão
cargo run

# Override específico
SERVER_PORT=8080 cargo run
```

### **Docker**

```dockerfile
# Dockerfile
COPY .env.production .env
CMD ["cargo", "run", "--release"]
```

### **Kubernetes**

```yaml
# ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: backend-pbx-config
data:
  ENVIRONMENT: "production"
  SERVER_PORT: "3000"
  LOG_LEVEL: "info"
```

### **CI/CD**

```yaml
# GitHub Actions
env:
  DATABASE_URL: ${{ secrets.DATABASE_URL }}
  JWT_SECRET: ${{ secrets.JWT_SECRET }}
  ENCRYPTION_KEY: ${{ secrets.ENCRYPTION_KEY }}
```

## 🧪 Testes

### **Executar com Configurações de Teste**

```bash
# Usar .env específico para testes
cp .env.example .env.test
ENVIRONMENT=test cargo test
```

### **Testes de Configuração**

```rust
#[test]
fn test_default_config_loading() {
    let config = AppConfig::load().expect("Should load default config");
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 3000);
}

#[test]
fn test_env_override() {
    env::set_var("SERVER_PORT", "8080");
    let config = AppConfig::load().expect("Should load with override");
    assert_eq!(config.server.port, 8080);
}
```

## 🚀 Próximos Passos

1. **Configurar seu .env** baseado no `.env.example`
2. **Executar a aplicação** com `cargo run`
3. **Testar diferentes ambientes** criando múltiplos arquivos .env
4. **Implementar secrets management** para produção
5. **Configurar monitoramento** com as métricas habilitadas

---

## 💡 Dicas Úteis

- Use `dotenv` para carregar variáveis em scripts externos
- Combine com `direnv` para carregamento automático por diretório
- Implemente rotação automática de chaves em produção
- Configure alertas para configurações inválidas
- Documente todas as variáveis customizadas no seu .env.example

**🎉 Configuração organizada, segura e flexível!**

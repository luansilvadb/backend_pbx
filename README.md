# 🚀 Backend PBX - Plataforma VoIP Multi-Tenant

**Backend modular em Rust para controle de servidor FreeSWITCH via ESL (Event Socket Library).**

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![API](https://img.shields.io/badge/API-REST-blue.svg)](http://localhost:8080/swagger-ui)
[![Docker](https://img.shields.io/badge/docker-ready-brightgreen.svg)](./Dockerfile)

---

## 🏗️ **Nova Arquitetura Modular**

### **Stack Tecnológica**
- **🦀 Rust** - Performance e segurança
- **🌐 Axum** - Framework web moderno e rápido  
- **📚 Swagger/OpenAPI** - Documentação automática da API
- **⚙️ YAML Config** - Configuração externa flexível
- **📊 Tracing** - Logging estruturado
- **🐳 Docker** - Containerização ready

### **Arquitetura em Camadas**
```
📱 API REST (Axum + Swagger)
    ↓
🔧 Service Layer (Call Manager)
    ↓
📡 FreeSWITCH Service (ESL)
    ↓
🖥️ FreeSWITCH Server
```

---

## 🚀 **Quick Start**

### **1. Executar Localmente**
```bash
# Clonar e compilar
git clone <repo>
cd backend_pbx
cargo build --release

# Executar (FreeSWITCH deve estar rodando)
cargo run
```

### **2. Executar com Docker**
```bash
# Build e execução
docker-compose up --build

# Ou apenas o backend (FreeSWITCH externo)
docker build -t backend_pbx .
docker run -p 8080:8080 backend_pbx
```

### **3. Acessar API**
- **🌐 API Base:** http://localhost:8080
- **📚 Swagger UI:** http://localhost:8080/swagger-ui
- **💚 Health Check:** http://localhost:8080/health

---

## 📋 **API Endpoints**

### **🔗 REST API v1**

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| `GET` | `/health` | Health check do serviço |
| `GET` | `/api/v1/status` | Status detalhado do sistema |
| `GET` | `/api/v1/statistics` | Estatísticas de chamadas |
| `GET` | `/api/v1/calls` | Listar chamadas ativas |
| `POST` | `/api/v1/calls` | Originar nova chamada |
| `GET` | `/api/v1/calls/{id}` | Detalhes de uma chamada |
| `POST` | `/api/v1/calls/{id}/hangup` | Encerrar chamada |

### **📖 Exemplos de Uso**

```bash
# Listar chamadas ativas
curl http://localhost:8080/api/v1/calls

# Originar nova chamada
curl -X POST http://localhost:8080/api/v1/calls \
  -H "Content-Type: application/json" \
  -d '{"from": "1000", "to": "1001", "timeout_seconds": 30}'

# Encerrar chamada
curl -X POST http://localhost:8080/api/v1/calls/{call_id}/hangup
```

---

## ⚙️ **Configuração**

### **Arquivo config.yaml**
```yaml
# Servidor HTTP
server:
  host: "127.0.0.1"
  port: 8080

# FreeSWITCH Event Socket
freeswitch:
  host: "127.0.0.1"
  port: 8021
  password: "ClueCon"
  timeout_seconds: 30

# Logging
logging:
  level: "info"
  format: "json"

# API
api:
  cors_origins: ["*"]
  rate_limit_requests_per_minute: 100
```

### **Variáveis de Ambiente**
```bash
# Override de configurações (formato: APP__SECTION__KEY)
export APP__SERVER__PORT=9090
export APP__FREESWITCH__HOST=192.168.1.100
export APP__LOGGING__LEVEL=debug
```

---

## 📁 **Estrutura do Projeto**

```
backend_pbx/
├── 📦 Cargo.toml              # Dependências e configuração
├── ⚙️ config.yaml            # Configuração da aplicação
├── 🐳 Dockerfile             # Container definition
├── 🐳 docker-compose.yml     # Stack completa
├── 🧪 teste_conexao.ps1      # Script de teste
└── 📂 src/
    ├── 🚀 main.rs            # Entry point
    ├── 📚 lib.rs             # Library exports
    ├── 📂 api/               # API Layer (REST + Swagger)
    │   ├── mod.rs
    │   ├── handlers/         # HTTP handlers
    │   └── middleware/       # Custom middlewares
    ├── 📂 config/            # Configuration management
    ├── 📂 models/            # Data models & types
    │   ├── call.rs          # Call entities
    │   └── error.rs         # Error handling
    └── 📂 services/          # Business logic
        ├── freeswitch.rs    # FreeSWITCH ESL abstraction
        └── call_manager.rs  # Call management logic
```

---

## 🔧 **Desenvolvimento**

### **Pré-requisitos**
- **Rust 1.75+** - `winget install Rustlang.Rust.MSVC`
- **FreeSWITCH** - Configurado conforme especificado
- **Docker** (opcional) - Para containerização

### **Teste Rápido**
```bash
# Verificar ambiente
.\teste_conexao.ps1

# Executar com logs detalhados
RUST_LOG=backend_pbx=debug,tower_http=debug cargo run

# Build release
cargo build --release
```

### **Configuração FreeSWITCH**

**Event Socket:** `conf/autoload_configs/event_socket.conf.xml`
```xml
<configuration name="event_socket.conf">
  <settings>
    <param name="listen-ip" value="127.0.0.1"/>
    <param name="listen-port" value="8021"/>
    <param name="password" value="ClueCon"/>
  </settings>
</configuration>
```

**Usuários SIP:** `conf/directory/default/1000.xml`
```xml
<user id="1000">
  <params>
    <param name="password" value="1234"/>
  </params>
</user>
```

---

## 📊 **Monitoramento**

### **Health Checks**
```bash
# Health básico
curl http://localhost:8080/health

# Status detalhado
curl http://localhost:8080/api/v1/status

# Estatísticas
curl http://localhost:8080/api/v1/statistics
```

### **Logs Estruturados**
```bash
# Configurar nível de log
export RUST_LOG=backend_pbx=info,tower_http=debug

# Logs em JSON (produção)
export APP__LOGGING__FORMAT=json
```

---

## 🐳 **Deploy**

### **Docker Compose (Recomendado)**
```bash
# Stack completa (Backend + FreeSWITCH)
docker-compose up -d

# Verificar status
docker-compose logs -f backend_pbx
```

### **Docker Standalone**
```bash
# Build
docker build -t backend_pbx:latest .

# Run
docker run -d \
  --name backend_pbx \
  -p 8080:8080 \
  -e APP__FREESWITCH__HOST=host.docker.internal \
  backend_pbx:latest
```

---

## 🚧 **Roadmap**

### **✅ Concluído**
- [x] Arquitetura modular escalável
- [x] API REST completa com Swagger
- [x] Configuração externa (YAML + env vars)
- [x] Logging estruturado
- [x] Containerização Docker
- [x] Health checks & monitoring

### **🔄 Em Desenvolvimento**
- [ ] Autenticação JWT
- [ ] Rate limiting avançado
- [ ] Metrics Prometheus
- [ ] Testes automatizados
- [ ] CI/CD pipeline

### **🎯 Próximas Features**
- [ ] WebSocket para eventos em tempo real
- [ ] Multi-tenancy
- [ ] Dashboard web admin
- [ ] Integração com databases
- [ ] Load balancing

---

## 🤝 **Contribuição**

1. Fork o projeto
2. Crie uma branch: `git checkout -b feature/nova-feature`
3. Commit: `git commit -m 'feat: adicionar nova feature'`
4. Push: `git push origin feature/nova-feature`
5. Abra um Pull Request

---

## 📄 **Licença**

Este projeto está sob licença MIT. Veja [LICENSE](LICENSE) para detalhes.

---

**🚀 Backend PBX - Transformando VoIP com Rust!**
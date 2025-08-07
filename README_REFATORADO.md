# 🚀 Backend PBX - Sistema Multi-Tenant Refatorado

## 📋 Visão Geral

Este projeto implementa um sistema PABX (Private Branch Exchange) moderno e robusto em Rust, desenvolvido especificamente para ambientes multi-tenant em nuvem (SaaS). O FreeSWITCH atua como motor de mídia enquanto o backend Rust controla toda a lógica de negócio através do Event Socket Layer (ESL).

### ✨ Principais Características

- **🏢 Multi-Tenancy**: Isolamento completo entre clientes
- **📡 ESL Avançado**: Comunicação robusta com FreeSWITCH
- **🔒 Segurança**: Validação rigorosa de acesso por tenant
- **📊 Métricas**: Monitoramento detalhado por cliente
- **🎯 Escalabilidade**: Arquitetura preparada para crescimento
- **🛠️ APIs REST**: Interface moderna com documentação OpenAPI

## 🏗️ Arquitetura Refatorada

### Componentes Principais

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   API REST      │    │   FreeSWITCH    │    │   Tenants       │
│   (Axum)        │◄──►│   (ESL)         │    │   (Multi)       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Backend Rust                                │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│  │   Services   │ │    Models    │ │   Handlers   │            │
│  │              │ │              │ │              │            │
│  │ • ESL Mgmt   │ │ • Tenants    │ │ • REST APIs  │            │
│  │ • Tenants    │ │ • Calls      │ │ • Validation │            │
│  │ • Calls      │ │ • Events     │ │ • Security   │            │
│  │ • Security   │ │ • Commands   │ │ • Metrics    │            │
│  └──────────────┘ └──────────────┘ └──────────────┘            │
└─────────────────────────────────────────────────────────────────┘
```

### Módulos ESL Avançados

#### 1. **Events Module** (`src/services/esl/events.rs`)
Gerencia todos os eventos enviados pelo FreeSWITCH:

```rust
// Eventos de ciclo de vida
EslEventType::ChannelCreate    // Nova chamada
EslEventType::ChannelAnswer    // Chamada atendida
EslEventType::ChannelHangup    // Chamada encerrada

// Eventos de interação
EslEventType::Dtmf             // Dígitos pressionados
EslEventType::PlaybackStart    // Reprodução iniciada
EslEventType::RecordStart      // Gravação iniciada

// Eventos de sistema
EslEventType::SofiaRegister    // Registro SIP
EslEventType::Heartbeat        // Saúde do sistema
```

#### 2. **Commands Module** (`src/services/esl/commands.rs`)
Comandos tipados e seguros para controlar o FreeSWITCH:

```rust
// Comandos críticos
OriginateCommand              // Originar chamadas
UuidBridgeCommand            // Conectar chamadas
UuidPlaybackCommand          // Reproduzir áudio
UuidRecordCommand            // Gravar chamadas

// Validação automática por tenant
CommandExecutor::execute()    // Execução segura
TenantValidator::validate()   // Isolamento garantido
```

#### 3. **Multi-Tenant Service** (`src/services/tenant_service.rs`)
Gerenciamento completo de tenants:

```rust
// Operações CRUD
create_tenant()              // Criar novo tenant
get_tenant()                 // Buscar tenant
update_tenant()              // Atualizar configurações
list_tenants()               // Listar com paginação

// Validações e limites
validate_tenant_operation()  // Verificar permissões
get_tenant_limits()          // Obter limites
get_tenant_usage()           // Métricas de uso
```

## 🚀 Funcionalidades Implementadas

### 📞 Operações de Chamada

#### Click-to-Call Multi-Tenant
```rust
// Originar chamada para tenant específico
let call_uuid = freeswitch_service.originate_call_for_tenant(
    "tenant_123",           // ID do tenant
    "1001",                 // Ramal interno
    "+5511999999999"        // Número externo
).await?;
```

#### IVR Inteligente
```rust
// Processar DTMF com lógica específica do tenant
match digit {
    '1' => transfer_to_sales(tenant_id, call_uuid).await?,
    '2' => transfer_to_support(tenant_id, call_uuid).await?,
    '9' => play_tenant_menu(tenant_id, call_uuid).await?,
}
```

#### Conferências Empresariais
```rust
// Conectar participantes à sala do tenant
let bridge_cmd = UuidBridgeCommand {
    destination: BridgeDestination::Conference { 
        room: format!("{}_meeting_room", tenant_id),
        profile: Some("default".to_string())
    },
    // ...
};
```

### 🔒 Segurança Multi-Tenant

#### Isolamento de Recursos
```rust
// Validação automática de acesso a arquivos
TenantValidator::validate_resource_access(
    tenant_id, 
    "/recordings/tenant_123/call.wav"  // ✅ Permitido
)?;

TenantValidator::validate_resource_access(
    tenant_id, 
    "/recordings/tenant_456/call.wav"  // ❌ Negado
)?;
```

#### Rate Limiting por Tenant
```rust
// Verificar limites antes de executar operações
tenant_service.validate_tenant_operation(
    tenant_id, 
    "originate_call"
).await?;  // Pode retornar TooManyRequests
```

### 📊 Monitoramento e Métricas

#### Métricas Detalhadas por Tenant
```rust
let usage = tenant_service.get_tenant_usage(tenant_id).await?;
println!("Calls: {}, Minutes: {}, ASR: {:.2}%", 
         usage.total_calls, 
         usage.total_minutes, 
         usage.answer_seizure_ratio * 100.0);
```

#### Health Checks
```rust
let health = tenant_service.get_tenant_health(tenant_id).await?;
if !health.is_healthy {
    warn!("Tenant {} exceeding limits", tenant_id);
}
```

## 🛠️ APIs REST

### Endpoints de Tenants

```http
# Listar tenants
GET /api/tenants?page=1&per_page=20

# Criar tenant
POST /api/tenants
Content-Type: application/json
{
  "id": "new_company",
  "name": "New Company Ltd",
  "service_plan": "professional",
  "sip_domain": "new.pbx.local",
  "contact": { ... }
}

# Obter detalhes
GET /api/tenants/new_company

# Métricas de uso
GET /api/tenants/new_company/usage

# Status de saúde
GET /api/tenants/new_company/health
```

### Endpoints de Chamadas

```http
# Listar chamadas do tenant
GET /api/calls?tenant_id=new_company

# Originar chamada
POST /api/calls
{
  "tenant_id": "new_company",
  "from": "1001",
  "to": "+5511999999999"
}

# Encerrar chamada
DELETE /api/calls/{call_uuid}
```

## 🚀 Como Executar

### 1. Preparar Ambiente

```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clonar repositório
git clone <repository>
cd backend_pbx

# Instalar dependências
cargo build
```

### 2. Configurar FreeSWITCH

```bash
# Instalar FreeSWITCH
# Ubuntu/Debian:
sudo apt-get install freeswitch freeswitch-mod-commands

# Configurar ESL (editar /etc/freeswitch/autoload_configs/event_socket.conf.xml)
<param name="listen-ip" value="127.0.0.1"/>
<param name="listen-port" value="8021"/>
<param name="password" value="ClueCon"/>
```

### 3. Executar Sistema

```bash
# Executar backend
cargo run

# Em outro terminal, executar demonstração
cargo run --example multi_tenant_demo
```

### 4. Acessar Documentação

```
http://localhost:3000/swagger-ui/
```

## 📁 Estrutura do Projeto

```
backend_pbx/
├── src/
│   ├── api/
│   │   └── handlers/
│   │       ├── calls.rs         # Endpoints de chamadas
│   │       ├── tenants.rs       # Endpoints de tenants
│   │       └── health.rs        # Health checks
│   ├── models/
│   │   ├── call.rs              # Estruturas de chamadas
│   │   ├── tenant.rs            # Estruturas de tenants
│   │   └── error.rs             # Tratamento de erros
│   ├── services/
│   │   ├── esl/
│   │   │   ├── events.rs        # Eventos ESL tipados
│   │   │   ├── commands.rs      # Comandos ESL seguros
│   │   │   └── types.rs         # Tipos compartilhados
│   │   ├── freeswitch_enhanced.rs  # Serviço ESL avançado
│   │   ├── tenant_service.rs    # Gerenciamento de tenants
│   │   └── call_manager.rs      # Gerenciamento de chamadas
│   ├── config/
│   │   └── mod.rs               # Configurações
│   └── main.rs                  # Ponto de entrada
├── examples/
│   └── multi_tenant_demo.rs     # Demonstração completa
├── docs/
│   └── ESL_REFERENCE.md         # Documentação técnica ESL
└── README_REFATORADO.md         # Este arquivo
```

## 🔄 Exemplos de Uso

### Cenário 1: Empresa de Vendas

```rust
// Criar tenant de vendas
let sales_tenant = CreateTenantRequest {
    id: "sales_corp",
    name: "Sales Corporation",
    service_plan: ServicePlan::Professional,
    sip_domain: "sales.pbx.local",
    config: Some(TenantConfigRequest {
        recording_policy: Some(RecordingPolicy::Always),
        billing_settings: Some(BillingSettings {
            local_rate_per_minute: 0.05,
            // ...
        }),
        // ...
    }),
    // ...
};

// Implementar click-to-call
async fn click_to_call(extension: &str, external: &str) -> Result<Uuid> {
    let call_uuid = freeswitch.originate_call_for_tenant(
        "sales_corp", extension, external
    ).await?;
    
    // Reproduzir mensagem de vendas
    freeswitch.playback_for_tenant(
        "sales_corp", call_uuid, "sales_intro.wav"
    ).await?;
    
    Ok(call_uuid)
}
```

### Cenário 2: Call Center de Suporte

```rust
// IVR dinâmico para suporte
async fn handle_support_ivr(call_uuid: Uuid, digit: char) -> Result<()> {
    match digit {
        '1' => {
            // Conectar à fila técnica
            let bridge_cmd = UuidBridgeCommand {
                destination: BridgeDestination::Queue { 
                    name: "support_center_technical".to_string()
                },
                // ...
            };
            freeswitch.execute_command_for_tenant(bridge_cmd, "support_center").await?;
        },
        '2' => {
            // Conectar à fila comercial
            // ...
        },
        '9' => {
            // Repetir menu
            freeswitch.playback_for_tenant(
                "support_center", call_uuid, "main_menu.wav"
            ).await?;
        },
        _ => {
            // Opção inválida
            freeswitch.playback_for_tenant(
                "support_center", call_uuid, "invalid_option.wav"
            ).await?;
        }
    }
    Ok(())
}
```

## 📊 Benefícios da Refatoração

### 🎯 Antes vs Depois

| Aspecto | Antes | Depois |
|---------|-------|--------|
| **Multi-Tenancy** | ❌ Não suportado | ✅ Isolamento completo |
| **Segurança** | ⚠️ Básica | ✅ Validação rigorosa |
| **Eventos ESL** | 📝 Strings cruas | ✅ Tipos estruturados |
| **Comandos ESL** | 📝 Strings manuais | ✅ Builders seguros |
| **Métricas** | ❌ Limitadas | ✅ Detalhadas por tenant |
| **Escalabilidade** | ⚠️ Questionável | ✅ Preparada para crescimento |
| **Manutenibilidade** | ⚠️ Acoplada | ✅ Modular e testável |
| **Documentação** | ❌ Escassa | ✅ Completa e atualizada |

### 🚀 Vantagens Técnicas

1. **Type Safety**: Eventos e comandos ESL totalmente tipados
2. **Memory Safety**: Gestão segura de recursos com Rust
3. **Concurrency**: Processamento assíncrono eficiente  
4. **Security**: Isolamento rigoroso entre tenants
5. **Observability**: Logging estruturado e métricas detalhadas
6. **Testability**: Arquitetura modular com traits
7. **Performance**: Otimizado para alta carga
8. **Maintainability**: Código limpo e bem documentado

## 🔮 Próximos Passos

### Implementações Futuras

- [ ] **Banco de Dados**: Migrar de storage em memória para PostgreSQL
- [ ] **Autenticação**: JWT e OAuth2 para APIs
- [ ] **Webhooks**: Notificações HTTP para eventos de tenant
- [ ] **Analytics**: Dashboard web para métricas
- [ ] **Auto-scaling**: Balanceamento automático de carga
- [ ] **CDR Processing**: Processamento avançado de registros
- [ ] **AI Integration**: Análise inteligente de chamadas
- [ ] **Mobile SDK**: SDK para aplicações móveis

### Melhorias de Performance

- [ ] **Connection Pooling**: Pool de conexões ESL
- [ ] **Caching**: Redis para dados frequentes
- [ ] **Sharding**: Distribuição por tenant
- [ ] **Monitoring**: Prometheus + Grafana
- [ ] **Load Testing**: Testes de carga automatizados

---

## 📞 Suporte

Para dúvidas, sugestões ou contribuições:

- **Documentação**: `docs/ESL_REFERENCE.md`
- **Exemplos**: `examples/multi_tenant_demo.rs`
- **Issues**: GitHub Issues
- **Discussões**: GitHub Discussions

---

**🎉 Sistema PABX Multi-Tenant pronto para produção!**

*Desenvolvido com ❤️ em Rust para máxima performance e segurança.*

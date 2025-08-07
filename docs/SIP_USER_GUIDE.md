# 📞 Guia de Usuários SIP - Integração com MicroSIP

Este guia explica como usar o sistema de usuários SIP implementado no Backend PBX para registrar e autenticar softphones como MicroSIP.

## ✅ **Funcionalidades Implementadas**

### 🔧 **Modelos e Estruturas**
- ✅ **SipUser**: Modelo completo de usuário SIP com extensões, senhas criptografadas e configurações
- ✅ **SipUserConfig**: Configurações avançadas (codecs, permissões, SRTP, etc.)
- ✅ **SipUserLimits**: Limites por usuário (chamadas simultâneas, minutos mensais, etc.)
- ✅ **SipRegistrationInfo**: Informações de registro em tempo real
- ✅ **SipUserUsageStats**: Estatísticas de uso e billing

### 🛠️ **Serviços**
- ✅ **SipUserService**: Gerenciamento completo de usuários SIP
- ✅ **Autenticação BCrypt**: Senhas criptografadas com bcrypt
- ✅ **Multi-tenancy**: Usuários isolados por tenant
- ✅ **Cache de registros**: Tracking de registros ativos

### 🌐 **API REST Completa**
- ✅ **CRUD de Usuários**: Criar, listar, atualizar, deletar usuários SIP
- ✅ **Gestão de Senhas**: Reset de senhas seguro
- ✅ **Controle de Status**: Ativar/desativar usuários
- ✅ **Registros SIP**: Lista de usuários registrados
- ✅ **Estatísticas**: Uso e métricas por usuário
- ✅ **Autenticação SIP**: Endpoint para FreeSWITCH validar credenciais

## 🚀 **Usuários Demo Criados**

O sistema automaticamente cria usuários demo no modo development:

### **Tenant: demo_company**
- **Extensão 1001**
  - Nome: João Silva
  - Email: joao@democompany.com
  - Senha: `demo123`

- **Extensão 1002**
  - Nome: Maria Santos  
  - Email: maria@democompany.com
  - Senha: `demo123`

### **Tenant: acme_corp**
- **Extensão 2001**
  - Nome: Alice Johnson
  - Email: alice@acme.corp
  - Senha: `acme123`

- **Extensão 2002**
  - Nome: Bob Wilson
  - Email: bob@acme.corp
  - Senha: `acme123`

## 📊 **Endpoints da API**

### **Gestão de Usuários SIP**

#### Listar Usuários
```http
GET /api/tenants/{tenant_id}/sip-users
```

#### Criar Usuário
```http
POST /api/tenants/{tenant_id}/sip-users
Content-Type: application/json

{
  "extension": "1003",
  "display_name": "Novo Usuário",
  "email": "usuario@empresa.com",
  "password": "senha123",
  "config": {
    "preferred_codec": "PCMU",
    "allow_external_calls": true,
    "allow_transfers": true,
    "allow_conferences": true,
    "auto_record": false,
    "registration_timeout": 3600,
    "force_srtp": false
  },
  "limits": {
    "max_concurrent_calls": 2,
    "max_call_duration": null,
    "monthly_minutes_limit": null,
    "allow_international": false,
    "allow_long_distance": false
  }
}
```

#### Buscar por Extensão
```http
GET /api/tenants/{tenant_id}/sip-users/extension/{extension}
```

#### Atualizar Usuário
```http
PUT /api/tenants/{tenant_id}/sip-users/{user_id}
```

#### Reset de Senha
```http
POST /api/tenants/{tenant_id}/sip-users/{user_id}/reset-password
Content-Type: application/json

{
  "new_password": "nova_senha123"
}
```

#### Alterar Status
```http
POST /api/tenants/{tenant_id}/sip-users/{user_id}/status
Content-Type: application/json

{
  "status": "active" // ou "inactive", "suspended", "blocked", "pending"
}
```

#### Listar Registros Ativos
```http
GET /api/tenants/{tenant_id}/sip-users/registrations
```

#### Estatísticas de Uso
```http
GET /api/tenants/{tenant_id}/sip-users/{user_id}/usage
```

### **Autenticação SIP (FreeSWITCH)**

```http
POST /api/sip/auth/{tenant_id}/{extension}
Content-Type: application/json

{
  "password": "senha_do_usuario",
  "ip": "192.168.1.100",
  "user_agent": "MicroSIP/3.21.3"
}
```

## 🔧 **Configuração para MicroSIP**

### **Passo 1: Acessar a API**

1. **Iniciar o Backend PBX**:
   ```bash
   cargo run
   ```

2. **Acessar Swagger UI**:
   ```
   http://127.0.0.1:3000/swagger-ui
   ```

3. **Verificar usuários disponíveis**:
   ```bash
   curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users
   ```

### **Passo 2: Configurar MicroSIP**

Para configurar MicroSIP com os usuários demo:

#### **Configuração para Extensão 1001 (demo_company)**
- **Account Name**: João Silva (1001)
- **SIP Server**: `demo.pbx.local` (ou IP do FreeSWITCH)
- **Username**: `1001`
- **Password**: `demo123`
- **Domain**: `demo.pbx.local`
- **Proxy**: (IP do FreeSWITCH, ex: 192.168.1.100:5060)

#### **Configuração para Extensão 1002 (demo_company)**
- **Account Name**: Maria Santos (1002)
- **SIP Server**: `demo.pbx.local`
- **Username**: `1002`
- **Password**: `demo123`
- **Domain**: `demo.pbx.local`
- **Proxy**: (IP do FreeSWITCH)

#### **Configuração para ACME Corp (extensões 2001, 2002)**
- **Domain**: `acme.pbx.local`
- **Username**: `2001` ou `2002`
- **Password**: `acme123`

### **Passo 3: Configurações Avançadas MicroSIP**

#### **Codec Settings**
- **Audio Codecs**: PCMU, PCMA, G.729 (conforme preferência do usuário)
- **Video**: Desabilitado (para configuração básica)

#### **Network Settings**
- **Transport**: UDP
- **Port**: 5060
- **STUN**: Configurar se necessário para NAT

#### **Security Settings**
- **SRTP**: Configurar se `force_srtp: true` no usuário
- **TLS**: Configurar se necessário

## 🔍 **Testando a Integração**

### **1. Verificar Usuários Criados**
```bash
curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users
```

### **2. Testar Autenticação**
```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"password":"demo123","ip":"192.168.1.100","user_agent":"MicroSIP/3.21.3"}' \
  http://127.0.0.1:3000/api/sip/auth/demo_company/1001
```

### **3. Verificar Registros Ativos**
```bash
curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users/registrations
```

### **4. Buscar Usuário por Extensão**
```bash
curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users/extension/1001
```

## 📋 **Status dos Usuários**

O sistema suporta os seguintes status:

- **`active`**: Usuário ativo, pode se registrar e fazer chamadas
- **`inactive`**: Usuário desativado temporariamente
- **`suspended`**: Usuário suspenso por violação de políticas
- **`blocked`**: Usuário bloqueado por excesso de tentativas
- **`pending`**: Usuário pendente de ativação

## 🔐 **Segurança**

### **Senhas**
- ✅ Criptografadas com BCrypt (cost factor 12)
- ✅ Validação de força mínima (6+ caracteres)
- ✅ Reset seguro via API

### **Autenticação**
- ✅ Endpoint dedicado para FreeSWITCH
- ✅ Validação de tenant e status
- ✅ Logging de tentativas

### **Isolamento Multi-Tenant**
- ✅ Usuários completamente isolados por tenant
- ✅ Domínios SIP únicos por tenant
- ✅ Configurações independentes

## 🚧 **Próximos Passos (TODO)**

- [ ] **Integração FreeSWITCH Directory**: Gerar arquivos XML automaticamente
- [ ] **Dialplan Automático**: Configuração de contextos por tenant
- [ ] **Presença e BLF**: Status de extensões em tempo real
- [ ] **Call Detail Records**: Integração com estatísticas de chamadas
- [ ] **Rate Limiting**: Proteção contra ataques de força bruta
- [ ] **Audit Log**: Log de todas as operações de usuários

## 🎯 **Conclusão**

O sistema de usuários SIP está **totalmente funcional** e pronto para integração com MicroSIP. Todas as funcionalidades básicas foram implementadas:

- ✅ **CRUD completo** de usuários SIP
- ✅ **Autenticação segura** com BCrypt
- ✅ **Multi-tenancy** completo
- ✅ **API REST** bem documentada
- ✅ **Swagger UI** para testes
- ✅ **Usuários demo** para testes
- ✅ **Gestão de registros** SIP
- ✅ **Controle de status** e permissões

O próximo passo seria integrar com o FreeSWITCH para geração automática de configurações, mas o sistema já está pronto para uso!

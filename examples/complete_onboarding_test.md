# 🎯 **Teste Completo de Onboarding - TechCorp Solutions**

Este exemplo demonstra o fluxo completo de onboarding de uma nova empresa no Backend PBX.

## 📋 **Cenário:**
- **Empresa:** TechCorp Solutions
- **Funcionários:** 5 pessoas  
- **Necessidades:** Central telefônica completa

## 🚀 **Fluxo Passo a Passo**

### **Passo 1: 🏢 Criar Tenant (Empresa)**

```bash
curl -X POST http://127.0.0.1:3000/api/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "id": "techcorp",
    "name": "TechCorp Solutions LTDA",
    "sip_domain": "techcorp.pbx.local",
    "default_context": "techcorp_context",
    "service_plan": "professional",
    "max_extensions": 25,
    "max_concurrent_calls": 10,
    "max_monthly_minutes": 5000,
    "max_recording_storage_mb": 2000
  }'
```

**Response Esperado:**
```json
{
  "id": "techcorp",
  "name": "TechCorp Solutions LTDA", 
  "status": "ACTIVE",
  "config": {
    "sip_domain": "techcorp.pbx.local",
    "default_context": "techcorp_context"
  },
  "service_plan": "professional",
  "created_at": "2025-08-07T03:00:00Z"
}
```

### **Passo 2: ✅ Validar Tenant Criado**

```bash
curl http://127.0.0.1:3000/api/tenants/techcorp
```

### **Passo 3: 📞 Criar Usuários SIP**

#### **3.1 CEO (extensão 2000)**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "2000",
    "display_name": "Carlos CEO",
    "email": "ceo@techcorp.com",
    "password": "ceo123",
    "config": {
      "allow_external_calls": true,
      "allow_international": true,
      "auto_record": false
    },
    "limits": {
      "max_concurrent_calls": 5,
      "allow_international": true,
      "allow_long_distance": true
    }
  }'
```

#### **3.2 Gerente (extensão 2001)**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "2001",
    "display_name": "Ana Gerente",
    "email": "gerente@techcorp.com",
    "password": "gerente123",
    "config": {
      "allow_external_calls": true,
      "allow_transfers": true,
      "auto_record": false
    },
    "limits": {
      "max_concurrent_calls": 3,
      "monthly_minutes_limit": 2000,
      "allow_long_distance": true
    }
  }'
```

#### **3.3 Recepção (extensão 2002)**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "2002",
    "display_name": "Maria Recepção",
    "email": "recepcao@techcorp.com",
    "password": "recepcao123",
    "config": {
      "allow_external_calls": true,
      "allow_transfers": true,
      "allow_conferences": true,
      "auto_record": true
    },
    "limits": {
      "max_concurrent_calls": 4,
      "monthly_minutes_limit": 3000
    }
  }'
```

#### **3.4 Vendas (extensão 2003)**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "2003",
    "display_name": "João Vendas",
    "email": "vendas@techcorp.com",
    "password": "vendas123",
    "config": {
      "allow_external_calls": true,
      "auto_record": true
    },
    "limits": {
      "max_concurrent_calls": 3,
      "monthly_minutes_limit": 2500,
      "allow_long_distance": true
    }
  }'
```

#### **3.5 Suporte (extensão 2004)**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "2004",
    "display_name": "Pedro Suporte",
    "email": "suporte@techcorp.com",
    "password": "suporte123",
    "config": {
      "allow_external_calls": false,
      "allow_transfers": true,
      "allow_conferences": true
    },
    "limits": {
      "max_concurrent_calls": 2,
      "monthly_minutes_limit": 1000
    }
  }'
```

### **Passo 4: 🔍 Verificar Sistema**

#### **4.1 Listar todos os usuários criados:**
```bash
curl http://127.0.0.1:3000/api/tenants/techcorp/sip-users
```

#### **4.2 Verificar saúde do tenant:**
```bash
curl http://127.0.0.1:3000/api/tenants/techcorp/health
```

#### **4.3 Obter estatísticas detalhadas:**
```bash
curl http://127.0.0.1:3000/api/tenants/techcorp/stats
```

### **Passo 5: 🧪 Testar Autenticação**

#### **5.1 Teste CEO:**
```bash
curl -X POST http://127.0.0.1:3000/api/sip/auth/techcorp/2000 \
  -H "Content-Type: application/json" \
  -d '{
    "password": "ceo123",
    "ip": "192.168.1.100",
    "user_agent": "MicroSIP/3.21.3"
  }'
```

#### **5.2 Teste Recepção:**
```bash
curl -X POST http://127.0.0.1:3000/api/sip/auth/techcorp/2002 \
  -H "Content-Type: application/json" \
  -d '{
    "password": "recepcao123", 
    "ip": "192.168.1.102",
    "user_agent": "MicroSIP/3.21.3"
  }'
```

#### **5.3 Verificar registros ativos:**
```bash
curl http://127.0.0.1:3000/api/tenants/techcorp/sip-users/registrations
```

### **Passo 6: 🔧 Configurar MicroSIP**

#### **CEO (2000):**
```
Account Name: Carlos CEO
Username: 2000
Password: ceo123
Domain: techcorp.pbx.local
SIP Server: techcorp.pbx.local
```

#### **Gerente (2001):**
```
Account Name: Ana Gerente
Username: 2001
Password: gerente123
Domain: techcorp.pbx.local
SIP Server: techcorp.pbx.local
```

#### **Recepção (2002):**
```
Account Name: Maria Recepção
Username: 2002
Password: recepcao123
Domain: techcorp.pbx.local
SIP Server: techcorp.pbx.local
```

#### **Vendas (2003):**
```
Account Name: João Vendas
Username: 2003
Password: vendas123
Domain: techcorp.pbx.local
SIP Server: techcorp.pbx.local
```

#### **Suporte (2004):**
```
Account Name: Pedro Suporte
Username: 2004
Password: suporte123
Domain: techcorp.pbx.local
SIP Server: techcorp.pbx.local
```

## 🎛️ **Gestão Pós-Onboarding**

### **Alterar Status de Usuário:**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/sip-users/{user_id}/status \
  -H "Content-Type: application/json" \
  -d '{"status": "inactive"}'
```

### **Reset de Senha:**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/sip-users/{user_id}/reset-password \
  -H "Content-Type: application/json" \
  -d '{"new_password": "nova_senha123"}'
```

### **Suspender Tenant:**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/status \
  -H "Content-Type: application/json" \
  -d '{"status": "SUSPENDED"}'
```

### **Adicionar Novo Usuário:**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/techcorp/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "2005",
    "display_name": "Lucas Financeiro",
    "email": "financeiro@techcorp.com",
    "password": "financeiro123"
  }'
```

## 📊 **Validação e Testes**

### **Validar ID antes de criar:**
```bash
curl http://127.0.0.1:3000/api/tenants/validate/techcorp2
```

### **Buscar usuário por extensão:**
```bash
curl http://127.0.0.1:3000/api/tenants/techcorp/sip-users/extension/2000
```

### **Atualizar dados do tenant:**
```bash
curl -X PUT http://127.0.0.1:3000/api/tenants/techcorp \
  -H "Content-Type: application/json" \
  -d '{
    "name": "TechCorp Solutions LTDA - Matriz",
    "service_plan": "enterprise"
  }'
```

## ✅ **Checklist de Sucesso**

- [ ] ✅ Tenant "techcorp" criado
- [ ] ✅ 5 usuários SIP criados (2000-2004)  
- [ ] ✅ Todos os usuários autenticando
- [ ] ✅ Configurações específicas aplicadas
- [ ] ✅ Limites individuais configurados
- [ ] ✅ Softphones configurados
- [ ] ✅ Testes de registro realizados
- [ ] ✅ Gestão de status funcionando
- [ ] ✅ APIs de estatísticas funcionando

## 🎯 **Resultado Final**

**TechCorp Solutions** estará com:
- ✅ **1 Tenant** configurado
- ✅ **5 Usuários SIP** ativos  
- ✅ **Hierarquia de permissões** adequada
- ✅ **Limites individualizados**
- ✅ **Sistema multi-tenant** isolado
- ✅ **Gestão completa** via API
- ✅ **Softphones** configurados
- ✅ **Monitoramento** ativo

**🎉 Onboarding Completo Realizado com Sucesso!**

# 🎯 **Exemplo Prático de Onboarding**

## **Situação:** Nova empresa "TechCorp" quer usar o sistema

## 📋 **Processo Passo a Passo**

### **Passo 1: 🏢 Tenant (Empresa) - PRIMEIRO**

**O que é:** O tenant representa a empresa/organização completa.

**Como funciona atualmente:**
- ✅ Os tenants `demo_company` e `acme_corp` já existem
- ❌ API de criação de tenants ainda não implementada (TODO)

**Informações do Tenant:**
```json
{
  "id": "demo_company",
  "name": "Demo Company LTDA", 
  "sip_domain": "demo.pbx.local",
  "max_extensions": 50,
  "max_concurrent_calls": 20
}
```

### **Passo 2: 📞 Usuários SIP - SEGUNDO**

**Agora criamos os usuários para o tenant existente:**

#### **2.1 Administrador da Empresa**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/demo_company/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "1000",
    "display_name": "Administrador TechCorp",
    "email": "admin@techcorp.com",
    "password": "admin123"
  }'
```

#### **2.2 Recepcionista**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/demo_company/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "1001", 
    "display_name": "Maria Recepção",
    "email": "recepcao@techcorp.com",
    "password": "recepcao123"
  }'
```

#### **2.3 Vendedor**
```bash
curl -X POST http://127.0.0.1:3000/api/tenants/demo_company/sip-users \
  -H "Content-Type: application/json" \
  -d '{
    "extension": "1002",
    "display_name": "João Vendas", 
    "email": "vendas@techcorp.com",
    "password": "vendas123"
  }'
```

### **Passo 3: 🔧 Configurar MicroSIP**

Para cada funcionário, configurar o MicroSIP:

#### **Administrador (1000):**
```
Account Name: Administrador TechCorp
Username: 1000
Password: admin123
Domain: demo.pbx.local
SIP Server: demo.pbx.local
```

#### **Recepção (1001):**
```
Account Name: Maria Recepção  
Username: 1001
Password: recepcao123
Domain: demo.pbx.local
SIP Server: demo.pbx.local
```

#### **Vendas (1002):**
```
Account Name: João Vendas
Username: 1002  
Password: vendas123
Domain: demo.pbx.local
SIP Server: demo.pbx.local
```

### **Passo 4: ✅ Testar Sistema**

#### **4.1 Listar usuários criados:**
```bash
curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users
```

#### **4.2 Testar autenticação:**
```bash
# Teste Admin
curl -X POST http://127.0.0.1:3000/api/sip/auth/demo_company/1000 \
  -H "Content-Type: application/json" \
  -d '{"password":"admin123","ip":"192.168.1.100"}'

# Teste Recepção  
curl -X POST http://127.0.0.1:3000/api/sip/auth/demo_company/1001 \
  -H "Content-Type: application/json" \
  -d '{"password":"recepcao123","ip":"192.168.1.101"}'
```

#### **4.3 Verificar registros:**
```bash
curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users/registrations
```

## 🎯 **Resumo do Fluxo**

### **ORDEM CORRETA:**
1. **🏢 PRIMEIRO: Tenant** (empresa/organização)
   - Define domínio SIP
   - Define limites globais  
   - Define contexto de discagem

2. **📞 SEGUNDO: Usuários SIP** (funcionários)
   - Pertencem ao tenant
   - Têm extensões únicas no tenant
   - Herdam configurações do tenant

### **Hierarquia:**
```
Tenant (demo_company)
├── Usuário 1000 (admin@techcorp.com)
├── Usuário 1001 (recepcao@techcorp.com)  
├── Usuário 1002 (vendas@techcorp.com)
└── Usuário 1003 (suporte@techcorp.com)
```

## 📝 **Estado Atual do Sistema**

### **✅ Funciona:**
- ✅ Criação de usuários SIP
- ✅ Autenticação de usuários
- ✅ Gestão de senhas
- ✅ Controle de status
- ✅ Listagem e busca
- ✅ API REST completa
- ✅ Swagger UI

### **🚧 TODO (próximos passos):**
- ❌ API para criar novos tenants
- ❌ Integração FreeSWITCH directory  
- ❌ Geração automática de dialplan
- ❌ Billing e relatórios

## 🛠️ **Como usar AGORA:**

### **Opção 1: Usar tenants demo**
Use `demo_company` ou `acme_corp` para testes e crie usuários neles.

### **Opção 2: Implementar API de tenants**
Implementar endpoint `POST /api/tenants` para criar novos tenants.

## 📞 **Exemplo Prático Completo**

**Empresa:** TechCorp Solutions  
**Tenant:** `demo_company` (reutilizando)  
**Funcionários:** 5 pessoas

### **1. Listar tenant disponível:**
```bash
curl http://127.0.0.1:3000/api/tenants
```

### **2. Criar usuários (5):**
```bash
# CEO
curl -X POST http://127.0.0.1:3000/api/tenants/demo_company/sip-users \
  -d '{"extension":"1000","display_name":"Carlos CEO","email":"ceo@techcorp.com","password":"ceo123"}'

# Gerente
curl -X POST http://127.0.0.1:3000/api/tenants/demo_company/sip-users \
  -d '{"extension":"1001","display_name":"Ana Gerente","email":"gerente@techcorp.com","password":"gerente123"}'

# Recepção
curl -X POST http://127.0.0.1:3000/api/tenants/demo_company/sip-users \
  -d '{"extension":"1002","display_name":"Maria Recepção","email":"recepcao@techcorp.com","password":"recepcao123"}'

# Vendas
curl -X POST http://127.0.0.1:3000/api/tenants/demo_company/sip-users \
  -d '{"extension":"1003","display_name":"João Vendas","email":"vendas@techcorp.com","password":"vendas123"}'

# Suporte
curl -X POST http://127.0.0.1:3000/api/tenants/demo_company/sip-users \
  -d '{"extension":"1004","display_name":"Pedro Suporte","email":"suporte@techcorp.com","password":"suporte123"}'
```

### **3. Configurar 5 MicroSIP:**
Cada funcionário configura seu MicroSIP com:
- Username: sua extensão (1000-1004)
- Password: sua senha
- Domain: demo.pbx.local

### **4. Testar funcionamento:**
```bash
curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users
```

**🎉 Pronto! Empresa TechCorp configurada com 5 usuários funcionando!**

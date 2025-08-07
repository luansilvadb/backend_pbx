# 🔧 Configuração do FreeSWITCH para Backend PBX

Este guia explica como configurar o FreeSWITCH para trabalhar com o Backend PBX e permitir que softphones como MicroSIP se conectem corretamente.

## 🎯 **Problema Resolvido**

O erro "Senha Incorreta" no MicroSIP era causado porque o FreeSWITCH estava recebendo `"temp_password"` em vez da senha real. Agora o sistema retorna a senha correta (`demo123`) para os usuários demo.

## 📋 **Configuração Necessária**

### **1. Configuração do mod_xml_curl**

Crie ou edite o arquivo `conf/autoload_configs/xml_curl.conf.xml`:

```xml
<configuration name="xml_curl.conf" description="cURL XML Gateway">
  <bindings>
    <binding name="directory">
      <param name="gateway-url" value="http://127.0.0.1:3000/api/freeswitch/directory" bindings="directory"/>
    </binding>
    <binding name="auth">
      <param name="gateway-url" value="http://127.0.0.1:3000/api/freeswitch/auth" bindings="directory"/>
    </binding>
  </bindings>
</configuration>
```

### **2. Configuração do Sofia Profile**

Edite o arquivo `conf/sip_profiles/internal.xml`:

```xml
<include>
  <profile name="internal">
    <settings>
      <!-- Configurações básicas -->
      <param name="sip-ip" value="0.0.0.0"/>
      <param name="sip-port" value="5060"/>
      <param name="context" value="default"/>
      <param name="dialplan" value="XML"/>
      
      <!-- Autenticação via XML -->
      <param name="auth-calls" value="true"/>
      <param name="directory" value="xml_curl"/>
      
      <!-- Configurações de domínio -->
      <param name="force-register-domain" value="demo_company"/>
      <param name="force-subscription-domain" value="demo_company"/>
      
      <!-- Configurações de registro -->
      <param name="accept-blind-reg" value="false"/>
      <param name="accept-blind-auth" value="false"/>
      
      <!-- Configurações de codec -->
      <param name="inbound-codec-prefs" value="PCMU,PCMA"/>
      <param name="outbound-codec-prefs" value="PCMU,PCMA"/>
      
      <!-- Configurações de NAT -->
      <param name="aggressive-nat-detection" value="true"/>
      <param name="apply-nat-acl" value="rfc1918.auto"/>
      
      <!-- Configurações de segurança -->
      <param name="disable-register" value="false"/>
      <param name="challenge-realm" value="auto_from"/>
    </settings>
  </profile>
</include>
```

### **3. Configuração do Dialplan**

Crie o arquivo `conf/dialplan/default.xml`:

```xml
<include>
  <context name="default">
    <!-- Chamadas internas -->
    <extension name="internal_calls">
      <condition field="destination_number" expression="^(1\d{3})$">
        <action application="set" data="domain_name=${sip_profile_name}"/>
        <action application="transfer" data="${destination_number} XML ${domain_name}"/>
      </condition>
    </extension>
    
    <!-- Chamadas externas (se permitido) -->
    <extension name="external_calls">
      <condition field="destination_number" expression="^(\d{10,})$">
        <action application="set" data="effective_caller_id_name=${outbound_caller_id_name}"/>
        <action application="set" data="effective_caller_id_number=${outbound_caller_id_number}"/>
        <action application="bridge" data="sofia/gateway/external/$1"/>
      </condition>
    </extension>
  </context>
</include>
```

## 🔧 **Configuração do MicroSIP**

### **Configuração para Usuário Demo (1001)**

```
Account Name: João Silva (1001)
SIP Server: 127.0.0.1
Username: 1001
Password: demo123
Domain: demo_company
Port: 5060
Transport: UDP
```

### **Configuração para Usuário Demo (1002)**

```
Account Name: Maria Santos (1002)
SIP Server: 127.0.0.1
Username: 1002
Password: demo123
Domain: demo_company
Port: 5060
Transport: UDP
```

## 🚀 **Testando a Configuração**

### **1. Verificar se o Backend está Rodando**

```bash
curl http://127.0.0.1:3000/health
```

### **2. Testar Autenticação via API**

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"password":"demo123","ip":"127.0.0.1","user_agent":"MicroSIP/3.21.3"}' \
  http://127.0.0.1:3000/api/sip/auth/demo_company/1001
```

### **3. Testar Directory Lookup**

```bash
curl -X POST "http://127.0.0.1:3000/api/freeswitch/directory?domain=demo_company&user=1001"
```

### **4. Verificar Logs do FreeSWITCH**

No console do FreeSWITCH, você deve ver:
```
[INFO] mod_xml_curl.c: XML curl requesting: http://127.0.0.1:3000/freeswitch/directory
[INFO] mod_xml_curl.c: XML curl response received
```

## 🔍 **Solução de Problemas**

### **Problema: "Senha Incorreta"**

**Causa**: FreeSWITCH não está recebendo a senha correta do backend.

**Solução**:
1. Verificar se o mod_xml_curl está carregado: `show modules | grep xml_curl`
2. Verificar se o backend está rodando na porta 3000
3. Testar o endpoint de directory: `curl -X POST "http://127.0.0.1:3000/api/freeswitch/directory?domain=demo_company&user=1001"`

### **Problema: "Usuário não encontrado"**

**Causa**: O domínio ou extensão não existe no backend.

**Solução**:
1. Verificar usuários criados: `curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users`
2. Verificar se o domínio está correto no MicroSIP
3. Verificar se a extensão existe no tenant correto

### **Problema: "Connection Refused"**

**Causa**: FreeSWITCH não consegue conectar ao backend.

**Solução**:
1. Verificar se o backend está rodando: `cargo run`
2. Verificar firewall/porta 3000
3. Verificar se a URL no xml_curl.conf.xml está correta

## 📊 **Logs e Monitoramento**

### **Logs do Backend**

```bash
RUST_LOG=backend_pbx=debug cargo run
```

### **Logs do FreeSWITCH**

```bash
# No console do FreeSWITCH
console_loglevel debug
```

### **Verificar Registros SIP**

```bash
curl http://127.0.0.1:3000/api/tenants/demo_company/sip-users/registrations
```

## ✅ **Configuração Completa**

Após seguir este guia:

1. ✅ **Backend PBX** rodando na porta 3000
2. ✅ **FreeSWITCH** configurado com mod_xml_curl
3. ✅ **MicroSIP** configurado com credenciais corretas
4. ✅ **Autenticação** funcionando via backend
5. ✅ **Chamadas** roteadas corretamente

## 🔐 **Segurança em Produção**

Para ambiente de produção:

1. **Usar HTTPS** para comunicação entre FreeSWITCH e backend
2. **Implementar autenticação** mais robusta
3. **Usar mod_auth_xml_curl** para validação de senhas
4. **Configurar firewall** adequadamente
5. **Usar senhas fortes** e únicas por usuário

## 🎯 **Próximos Passos**

1. **Implementar mod_auth_xml_curl** para autenticação segura
2. **Adicionar suporte a múltiplos domínios**
3. **Implementar presença e BLF**
4. **Adicionar gravação de chamadas**
5. **Implementar relatórios e estatísticas**

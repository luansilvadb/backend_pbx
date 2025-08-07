# 🔧 Script de Configuração do FreeSWITCH para Backend PBX
# Este script configura o FreeSWITCH para usar o backend via mod_xml_curl

Write-Host "🔧 Configurando FreeSWITCH para Backend PBX..." -ForegroundColor Green

# Caminho do FreeSWITCH (ajuste conforme necessário)
$FREESWITCH_PATH = "C:\Program Files\FreeSWITCH"
$CONFIG_PATH = "$FREESWITCH_PATH\conf"

Write-Host "📁 Verificando instalação do FreeSWITCH..." -ForegroundColor Yellow

if (-not (Test-Path $CONFIG_PATH)) {
    Write-Host "❌ Diretório de configuração do FreeSWITCH não encontrado: $CONFIG_PATH" -ForegroundColor Red
    Write-Host "💡 Ajuste a variável FREESWITCH_PATH no script se necessário" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ FreeSWITCH encontrado em: $FREESWITCH_PATH" -ForegroundColor Green

# 1. Criar configuração do mod_xml_curl
Write-Host "📝 Criando configuração do mod_xml_curl..." -ForegroundColor Yellow

$XML_CURL_CONFIG = @"
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
"@

$XML_CURL_PATH = "$CONFIG_PATH\autoload_configs\xml_curl.conf.xml"
$XML_CURL_CONFIG | Out-File -FilePath $XML_CURL_PATH -Encoding UTF8

Write-Host "✅ Configuração do mod_xml_curl criada: $XML_CURL_PATH" -ForegroundColor Green

# 2. Configurar Sofia Profile
Write-Host "📝 Configurando Sofia Profile..." -ForegroundColor Yellow

$SOFIA_CONFIG = @"
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
      <param name="force-register-domain" value="127.0.0.1"/>
      <param name="force-subscription-domain" value="127.0.0.1"/>
      
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
"@

$SOFIA_PATH = "$CONFIG_PATH\sip_profiles\internal.xml"
$SOFIA_CONFIG | Out-File -FilePath $SOFIA_PATH -Encoding UTF8

Write-Host "✅ Sofia Profile configurado: $SOFIA_PATH" -ForegroundColor Green

# 3. Configurar Dialplan
Write-Host "📝 Configurando Dialplan..." -ForegroundColor Yellow

$DIALPLAN_CONFIG = @"
<include>
  <context name="default">
    <!-- Chamadas internas -->
    <extension name="internal_calls">
      <condition field="destination_number" expression="^(1\d{3})$">
        <action application="set" data="domain_name=`${sip_profile_name}`"/>
        <action application="transfer" data="`${destination_number}` XML `${domain_name}`"/>
      </condition>
    </extension>
    
    <!-- Chamadas externas (se permitido) -->
    <extension name="external_calls">
      <condition field="destination_number" expression="^(\d{10,})$">
        <action application="set" data="effective_caller_id_name=`${outbound_caller_id_name}`"/>
        <action application="set" data="effective_caller_id_number=`${outbound_caller_id_number}`"/>
        <action application="bridge" data="sofia/gateway/external/`$1`"/>
      </condition>
    </extension>
  </context>
</include>
"@

$DIALPLAN_PATH = "$CONFIG_PATH\dialplan\default.xml"
$DIALPLAN_CONFIG | Out-File -FilePath $DIALPLAN_PATH -Encoding UTF8

Write-Host "✅ Dialplan configurado: $DIALPLAN_PATH" -ForegroundColor Green

# 4. Verificar se o mod_xml_curl está carregado
Write-Host "🔍 Verificando se o mod_xml_curl está carregado..." -ForegroundColor Yellow

Write-Host "📋 Próximos passos:" -ForegroundColor Cyan
Write-Host "1. Reinicie o FreeSWITCH" -ForegroundColor White
Write-Host "2. No console do FreeSWITCH, execute: show modules | grep xml_curl" -ForegroundColor White
Write-Host "3. Se não estiver carregado, execute: load mod_xml_curl" -ForegroundColor White
Write-Host "4. Teste a conexão do MicroSIP" -ForegroundColor White

Write-Host "`n🎯 Configuração do MicroSIP:" -ForegroundColor Cyan
Write-Host "Account Name: João Silva (1001)" -ForegroundColor White
Write-Host "SIP Server: 127.0.0.1" -ForegroundColor White
Write-Host "Username: 1001" -ForegroundColor White
Write-Host "Password: demo123" -ForegroundColor White
Write-Host "Domain: 127.0.0.1" -ForegroundColor White
Write-Host "Port: 5060" -ForegroundColor White
Write-Host "Transport: UDP" -ForegroundColor White

Write-Host "`n✅ Configuração concluída!" -ForegroundColor Green
Write-Host "🔄 Reinicie o FreeSWITCH para aplicar as mudanças" -ForegroundColor Yellow

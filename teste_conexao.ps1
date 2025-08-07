# 🔍 Teste Rápido - Backend PBX

Write-Host "🚀 Testando Backend PBX..." -ForegroundColor Cyan

# Verificar Rust
try {
    rustc --version | Out-Null
    Write-Host "✅ Rust OK" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust não encontrado!" -ForegroundColor Red
    exit 1
}

# Verificar FreeSWITCH
$connection = Test-NetConnection 127.0.0.1 -Port 8021 -WarningAction SilentlyContinue
if ($connection.TcpTestSucceeded) {
    Write-Host "✅ FreeSWITCH OK (porta 8021)" -ForegroundColor Green
} else {
    Write-Host "❌ FreeSWITCH não acessível" -ForegroundColor Red
    exit 1
}

# Compilar
if ((cargo build 2>&1; $LASTEXITCODE -eq 0)) {
    Write-Host "✅ Compilação OK" -ForegroundColor Green
} else {
    Write-Host "❌ Erro na compilação" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 Tudo pronto! Execute: cargo run" -ForegroundColor Green

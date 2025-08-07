# 🐳 Dockerfile - Backend PBX
# Multi-stage build para otimizar tamanho da imagem

# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Instalar dependências de sistema necessárias
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copiar manifesto para cache de dependências
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cache layer)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copiar código fonte
COPY src ./src

# Build aplicação
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Instalar dependências de runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Criar usuário não-privilegiado
RUN groupadd -r backend_pbx && useradd -r -g backend_pbx backend_pbx

WORKDIR /app

# Copiar binário
COPY --from=builder /app/target/release/backend_pbx ./
COPY config.yaml ./

# Configurar permissões
RUN chown -R backend_pbx:backend_pbx /app
USER backend_pbx

# Expor porta
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Executar aplicação
CMD ["./backend_pbx"]

# GEMINI Analysis: backend_pbx

## Project Overview

This project is a modular, multi-tenant backend for a VoIP platform, built using Rust. It's designed to control a FreeSWITCH server via its Event Socket Library (ESL). The architecture is layered, with a clear separation of concerns between the API, service logic, and FreeSWITCH communication.

**Key Technologies:**

*   **Rust:** For performance and safety.
*   **Axum:** A modern and fast web framework for the REST API.
*   **Tokio:** Asynchronous runtime.
*   **Serde:** For serialization and deserialization (JSON/YAML).
*   **Utoipa:** For automatic OpenAPI/Swagger documentation generation from code.
*   **Docker:** For containerization.
*   **FreeSWITCH:** The underlying VoIP platform.

**Architecture:**

The application follows a clean, layered architecture:

1.  **API Layer (Axum):** Exposes a RESTful API for managing the PBX. It uses `utoipa` to generate live, accurate Swagger/OpenAPI documentation directly from the Rust code, ensuring the documentation is always in sync.
2.  **Service Layer:** Contains the core business logic. This includes a `CallManager` to handle call operations, and services for managing tenants and SIP users.
3.  **FreeSWITCH Service:** A dedicated module that abstracts the complexities of communicating with the FreeSWITCH ESL.
4.  **Configuration:** A robust configuration system that loads settings from a `config.yaml` file and can be overridden by environment variables.

## Building and Running

### Local Development

1.  **Prerequisites:**
    *   Rust 1.75+
    *   A running FreeSWITCH instance.

2.  **Build:**
    ```bash
    cargo build --release
    ```

3.  **Run:**
    ```bash
    cargo run
    ```
    The application will connect to the FreeSWITCH instance specified in `config.yaml` or environment variables.

### Docker

The project is fully containerized and includes a `docker-compose.yml` for easy setup.

1.  **Build and Run with Docker Compose (Recommended):**
    This will start both the `backend_pbx` and a `freeswitch` container.
    ```bash
    docker-compose up --build
    ```

2.  **Run Standalone Backend:**
    If you have an external FreeSWITCH instance.
    ```bash
    docker build -t backend_pbx .
    docker run -p 8080:8080 backend_pbx
    ```

### Accessing the API

*   **API Base URL:** `http://localhost:8080`
*   **Swagger UI:** `http://localhost:8080/swagger-ui`
*   **Health Check:** `http://localhost:8080/health`

## Development Conventions

*   **Code-First API:** The API documentation is generated directly from the Rust code using `utoipa`. This means the single source of truth is the code itself, and there's no need to maintain separate OpenAPI/Swagger files.
*   **Configuration:** Configuration is managed via `config.yaml` and can be overridden with environment variables (e.g., `APP__SERVER__PORT=9090`).
*   **Logging:** The application uses the `tracing` library for structured logging. Log levels and formats can be configured.
*   **Modularity:** The codebase is organized into distinct modules (`api`, `services`, `models`, `config`) to promote separation of concerns and maintainability.
*   **Error Handling:** The project uses the `anyhow` and `thiserror` crates for robust error handling.
*   **Asynchronous:** The entire application is built on the Tokio async runtime.

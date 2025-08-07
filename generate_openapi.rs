use backend_pbx::api::ApiDoc;
use std::fs;
use utoipa::OpenApi;

fn main() {
    let doc = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&doc).expect("Failed to serialize OpenAPI");
    fs::write("openapi_generated.json", json).expect("Failed to write file");
    println!("OpenAPI gerado com sucesso em: openapi_generated.json");
}

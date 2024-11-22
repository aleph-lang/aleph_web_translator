use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::env;
use log::{info, error};
use aleph_ollama::translate_code_from_string;

#[derive(Deserialize)]
struct TranslateRequest {
    source_code: String,
    target_language: Option<String>,
}

#[derive(Serialize)]
struct TranslateResponse {
    translated_code: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn translate(body: web::Json<TranslateRequest>) -> impl Responder {
    let target_language = body.target_language.clone().unwrap_or_else(|| "Java".to_string());
    let source_code = body.source_code.clone();
    
    info!("Translating code to {}", target_language);
    
    // Utiliser web::block pour exécuter le code bloquant dans un thread séparé
    let result = web::block(move || {
        translate_code_from_string(&source_code, &target_language)
    })
    .await;

    match result {
        Ok(Ok(translated_code)) => {
            info!("Translation successful");
            HttpResponse::Ok().json(TranslateResponse {
                translated_code,
            })
        }
        Ok(Err(error)) => {
            error!("Translation error: {}", error);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: error.to_string(),
            })
        }
        Err(e) => {
            error!("Blocking error: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Server error: {}", e),
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init();

    // Get the port from the environment variable or use 3030 as default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    info!("Starting server on port {}", port);

    // Start the server
    HttpServer::new(|| {
        App::new()
            .route("/translate", web::post().to(translate))
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

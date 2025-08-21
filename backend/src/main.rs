use actix::prelude::*;
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use chat::{chat_endpoint, ModelActor};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running...");
    env_logger::init();

    let base_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".into());

    let model_addr = ModelActor {
        base_url,
        client: reqwest::Client::new(),
    }
    .start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(model_addr.clone()))
            .route("/api/chat", web::post().to(chat_endpoint))
            // Serve your static frontend (adjust path as needed)
            .service(Files::new("/", "../frontend").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

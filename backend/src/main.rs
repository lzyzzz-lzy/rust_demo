mod api;
mod handlers;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载环境变量
    dotenv().ok();
    env_logger::init();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    println!("Server running at http://{}:{}", host, port);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            // 文件上传路由
            .service(
                web::resource("/api/upload")
                    .route(web::post().to(handlers::upload::upload_file))
            )
            // 文本处理路由 - 流式版本（用于分析、大纲等）
            .service(
                web::resource("/api/process/stream")
                    .route(web::get().to(handlers::process::process_text_stream))
            )
            // 应用修改路由（仅用于大纲、润色、续写）
            .service(
                web::resource("/api/apply")
                    .route(web::post().to(handlers::process::apply_changes))
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}

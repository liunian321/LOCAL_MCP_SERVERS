use std::{path::Path, process::exit};

use tokio::{fs, net::TcpListener};

use axum::Router;

mod config;
mod router;
mod tools;
use config::Config;

use crate::router::router::init_bind_router;

#[tokio::main]
async fn main() {
    let config_path = Path::new("config.yaml");
    let config_str = match fs::read_to_string(config_path).await {
        Ok(config) => config,
        Err(e) => {
            // 如果没有 config.yaml 直接退出
            eprintln!("Error reading config file: {}", e);
            exit(1);
        }
    };
    let config: Config = match serde_yaml::from_str(&config_str) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            exit(1);
        }
    };

    let app = init_bind_router(Router::new());
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.listen_port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    // use RUST_LOG=info cargo run
}

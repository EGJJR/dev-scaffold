use std::env;
use std::net::SocketAddr;

use {{ crate_name }}::app;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    if env::var("SECRET_KEY").is_err() {
        eprintln!("error: SECRET_KEY is required");
        std::process::exit(1);
    }

    let bind = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".to_string());
    let addr: SocketAddr = bind.parse().unwrap_or_else(|_| {
        eprintln!("error: invalid BIND_ADDR");
        std::process::exit(1);
    });

    let listener = TcpListener::bind(addr).await.unwrap_or_else(|err| {
        eprintln!("error: failed to bind {addr}: {err}");
        std::process::exit(1);
    });

    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, get_service},
};
use socketioxide::SocketIo;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::services::fs::ServeDir;
use tracing_subscriber::FmtSubscriber;
use utils::{FRONTEND_PORT, SOCKETIO_PORT, ServerConfig, ServerInfo};

// Import our modules
mod media_manager;
mod socket_io;
mod utils;

use socket_io::on_connect;

const FRONTEND_PATH: &str = "../client/dist";

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::default()).ok();
    let server_config = ServerConfig::new();

    let config_for_react = server_config.clone();
    let config_for_service = server_config.clone();

    let t0 = tokio::spawn(async move { serve_react_app(config_for_react).await });
    let t1 = tokio::spawn(async move { serve_socket_io(config_for_service).await });

    let _ = tokio::join!(t0, t1);
}

// Handler for server-info endpoint
async fn server_info_handler(State(config): State<ServerConfig>) -> Json<ServerInfo> {
    let socketio_url = config.get_url().await;
    Json(ServerInfo { socketio_url })
}

async fn serve_react_app(config: ServerConfig) -> Result<()> {
    let react_app = get_service(ServeDir::new(FRONTEND_PATH))
        .handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "Static file error") });

    let app = axum::Router::new()
        .route("/server-info", get(server_info_handler))
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive())
        .with_state(config)
        .fallback_service(react_app);

    let (listener, actual_port) = utils::try_bind(FRONTEND_PORT).await?;

    if actual_port != FRONTEND_PORT {
        println!(
            "Frontend port {} was unavailable, using port {} instead",
            FRONTEND_PORT, actual_port
        );
    }

    utils::print_urls("Frontend", actual_port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_socket_io(config: ServerConfig) -> Result<()> {
    let (layer, io) = SocketIo::new_layer();
    io.ns("/", on_connect);

    let layer = ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(layer);

    let app = Router::new()
        .layer(CorsLayer::permissive())
        .route("/health", get(|| async { "OK" }))
        .layer(layer);

    let (listener, actual_port) = utils::try_bind(SOCKETIO_PORT).await?;

    // Update the shared configuration with the actual Socket.IO port
    // Use the first network IP if available, otherwise use localhost
    let host = utils::get_local_ips()
        .first()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| String::from("localhost"));

    config.set_info(host.clone(), actual_port).await;

    if actual_port != SOCKETIO_PORT {
        println!(
            "SocketIO port {} was unavailable, using port {} instead",
            SOCKETIO_PORT, actual_port
        );
    }

    utils::print_urls("SocketIO", actual_port);

    axum::serve(listener, app).await?;
    Ok(())
}

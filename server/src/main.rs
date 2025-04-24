use anyhow::Result;
use axum::{
    Router,
    http::StatusCode,
    routing::{get, get_service},
};
use socketioxide::SocketIo;
use std::path::Path;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::services::fs::ServeDir;
use tracing_subscriber::FmtSubscriber;
use utils::{FRONTEND_PORT, SOCKETIO_PORT};

// Import our modules
mod media_manager;
mod socket_io;
mod utils;

use socket_io::on_connect;

#[tokio::main]
async fn main() {
    let t0 = tokio::task::spawn(async move { serve_react_app().await });
    let t1 = tokio::task::spawn(async move { serve_socket_io().await });
    let _ = tokio::join!(t0, t1);
}

async fn serve_react_app() -> Result<()> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let client_dist_path = Path::new("../client/dist");
    let react_app = get_service(ServeDir::new(client_dist_path))
        .handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "Static file error") });

    let app = axum::Router::new()
        .layer(CorsLayer::permissive())
        .route("/health", get(|| async { "OK" }))
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

async fn serve_socket_io() -> Result<()> {
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

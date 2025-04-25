use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, get_service},
};
use clap::Parser;
use socketioxide::SocketIo;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing_subscriber::FmtSubscriber;
use utils::{DEFAULT_FRONTEND_PORT, DEFAULT_SOCKETIO_PORT, ServerConfig, ServerInfo};

// Import our modules
mod media_manager;
mod socket_io;
mod utils;

use socket_io::on_connect;

/// Media Broadcast CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable the frontend server
    #[arg(long, short, default_value_t = false)]
    frontend: bool,

    #[arg(long, short = 'd', default_value_t = FRONTEND_DIR.to_string())]
    /// Path to the frontend files directory
    frontend_directory: String,

    /// Port for the frontend server
    #[arg(long, default_value_t = DEFAULT_FRONTEND_PORT)]
    frontend_port: u16,

    /// Port for the Socket.IO server
    #[arg(long, default_value_t = DEFAULT_SOCKETIO_PORT)]
    socketio_port: u16,
}

const FRONTEND_DIR: &str = "client/dist";

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(FmtSubscriber::default()).ok();
    let args = Args::parse();

    let config = ServerConfig::new(args.socketio_port);

    let config_for_socketio = config.clone();
    let server_task = tokio::spawn(async move {
        let port = args.frontend_port;
        if let Err(e) = serve_socket_io(config_for_socketio, port).await {
            tracing::error!("Socket.IO server error: {}", e);
        }
    });

    if args.frontend {
        let port = args.frontend_port;
        let dir = args.frontend_directory;

        let task = tokio::spawn(async move {
            if let Err(e) = serve_react_app(config, port, dir).await {
                eprintln!("Frontend service error: {}", e);
            }
        });
        task.await.ok();
    }

    server_task.await.ok();

    Ok(())
}

// Handler for server-info endpoint
async fn server_info_handler(State(config): State<ServerConfig>) -> Json<ServerInfo> {
    let socketio_url = config.get_url().await;
    Json(ServerInfo { socketio_url })
}

async fn serve_react_app(config: ServerConfig, port: u16, frontend_dir: String) -> Result<()> {
    tracing::debug!("Serving frontend from directory: {}", frontend_dir);

    let react_app = get_service(ServeDir::new(frontend_dir))
        .handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "Static file error") });

    let app = axum::Router::new()
        .route("/server-info", get(server_info_handler))
        .route("/health", get(|| async { "OK" }))
        .fallback_service(react_app)
        .with_state(config.clone());

    let (listener, actual_port) = utils::try_bind(port).await?;

    if actual_port != port {
        println!(
            "Frontend port {} was unavailable, using port {} instead",
            port, actual_port
        );
    }

    utils::print_urls("Frontend", actual_port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_socket_io(config: ServerConfig, port: u16) -> Result<()> {
    let (layer, io) = SocketIo::new_layer();
    io.ns("/", on_connect);

    let layer = ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(layer);

    let app = Router::new()
        .layer(CorsLayer::permissive())
        .route("/health", get(|| async { "OK" }))
        .layer(layer);

    let (listener, actual_port) = utils::try_bind(port).await?;

    // Update the shared configuration with the actual Socket.IO port
    // Use the first network IP if available, otherwise use localhost
    let host = utils::get_local_ips()
        .first()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| String::from("localhost"));

    config.set_info(host.clone(), actual_port).await;

    if actual_port != port {
        println!(
            "SocketIO port {} was unavailable, using port {} instead",
            port, actual_port
        );
    }

    utils::print_urls("SocketIO", actual_port);

    axum::serve(listener, app).await?;
    Ok(())
}

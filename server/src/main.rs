use anyhow::Result;
use axum::routing::get;
use socketioxide::SocketIo;
use socketioxide::extract::SocketRef;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing_subscriber::FmtSubscriber;

// Import our modules
mod media_manager;
mod socket_io;

use socket_io::on_connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (layer, io) = SocketIo::new_layer();

    io.ns("/ws", move |socket: SocketRef| {
        on_connect(socket.clone(), socket.clone())
    });

    let layer = ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(layer);

    let app = axum::Router::new()
        .route("/", get(|| async { "Media Server Running" }))
        .layer(layer);

    println!("Starting media server on 0.0.0.0:3001");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

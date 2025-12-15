mod storage;
mod service_impl;

use axum::{
    extract::{Request, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use hindsight_protocol::*;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use rapace::RpcSession;
use std::sync::Arc;
use std::time::Duration;

use crate::service_impl::HindsightServiceImpl;
use crate::storage::TraceStore;

pub async fn run_server(host: impl Into<String>, http_port: u16, tcp_port: u16, ttl_secs: u64) -> anyhow::Result<()> {
    let host = host.into();
    tracing::info!("🔍 Hindsight server starting");

    let store = TraceStore::new(Duration::from_secs(ttl_secs));
    let service = Arc::new(HindsightServiceImpl::new(store));

    // Spawn raw TCP server on port 1991 (for clients that want to skip HTTP handshake)
    let service_tcp = service.clone();
    let host_tcp = host.clone();
    tokio::spawn(async move {
        if let Err(e) = serve_tcp(&host_tcp, tcp_port, service_tcp).await {
            tracing::error!("TCP server error: {}", e);
        }
    });

    // Serve unified HTTP server on port 1990
    // Handles: HTTP GET, WebSocket upgrade, Rapace upgrade
    serve_http_unified(&host, http_port, service).await?;

    Ok(())
}

/// Serve Rapace RPC over TCP (for native clients)
async fn serve_tcp(
    host: &str,
    port: u16,
    service: Arc<HindsightServiceImpl>,
) -> anyhow::Result<()> {
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("📡 Rapace TCP server listening on {}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        tracing::info!("New TCP connection from {}", peer_addr);

        let service = service.clone();
        tokio::spawn(async move {
            let transport = Arc::new(rapace::transport::StreamTransport::new(stream));

            // IMPORTANT: No tracer attached! (Prevents infinite loop)
            let session = Arc::new(RpcSession::new(transport));

            // Create dispatcher function
            session.set_dispatcher(move |_channel_id, method_id, payload| {
                let service_impl = service.as_ref().clone();
                Box::pin(async move {
                    let server = HindsightServiceServer::new(service_impl);
                    server.dispatch(method_id, &payload).await
                })
            });

            if let Err(e) = session.run().await {
                tracing::error!("TCP session error: {}", e);
            }
        });
    }
}

/// Unified HTTP server on port 1990
/// Handles: HTTP GET /, WebSocket upgrade, Rapace upgrade
async fn serve_http_unified(
    host: &str,
    port: u16,
    service: Arc<HindsightServiceImpl>,
) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get({
            let service = service.clone();
            move |headers: HeaderMap, ws: Option<WebSocketUpgrade>, req: Request| {
                handle_root(headers, ws, req, service.clone())
            }
        }))
        .with_state(service);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🌐 Unified server listening on http://{}", addr);
    tracing::info!("  - HTTP GET / → Web UI");
    tracing::info!("  - Upgrade: websocket → WebSocket (for WASM clients)");
    tracing::info!("  - Upgrade: rapace → Raw Rapace TCP (for native clients)");

    axum::serve(listener, app).await?;
    Ok(())
}

/// Handle requests to "/" - detect upgrade type or serve HTML
async fn handle_root(
    headers: HeaderMap,
    ws: Option<WebSocketUpgrade>,
    req: Request,
    service: Arc<HindsightServiceImpl>,
) -> Response {
    // Check for Upgrade header
    let upgrade = headers
        .get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_lowercase());

    match upgrade.as_deref() {
        Some("websocket") => {
            // WebSocket upgrade
            if let Some(ws) = ws {
                ws.on_upgrade(move |socket| handle_websocket(socket, service))
                    .into_response()
            } else {
                (StatusCode::BAD_REQUEST, "WebSocket upgrade failed").into_response()
            }
        }
        Some("rapace") => {
            // Rapace upgrade - manual handling
            handle_rapace_upgrade(req, service).await.into_response()
        }
        _ => {
            // Normal HTTP - serve HTML
            let html = include_str!("ui/index.html");
            Html(html).into_response()
        }
    }
}

/// Handle WebSocket upgrade (for WASM clients)
async fn handle_websocket(
    _socket: axum::extract::ws::WebSocket,
    _service: Arc<HindsightServiceImpl>,
) {
    tracing::info!("New WebSocket connection");

    // TODO: Actually create the Rapace WebSocket transport here
    // For now, this is a placeholder - we'll need to properly bridge
    // the Axum WebSocket to rapace-transport-websocket
    tracing::warn!("WebSocket Rapace transport not yet implemented in unified server");
}

/// Handle Rapace HTTP upgrade (for native clients)
async fn handle_rapace_upgrade(
    mut req: Request,
    service: Arc<HindsightServiceImpl>,
) -> Response {
    // Extract the upgrade future from the request
    let upgrade = hyper::upgrade::on(&mut req);

    // Spawn task to handle the upgraded connection
    tokio::spawn(async move {
        match upgrade.await {
            Ok(upgraded) => {
                tracing::info!("Rapace HTTP upgrade successful");
                handle_rapace_connection(upgraded, service).await;
            }
            Err(e) => {
                tracing::error!("Rapace upgrade failed: {}", e);
            }
        }
    });

    // Return 101 Switching Protocols response
    Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header("Upgrade", "rapace")
        .header("Connection", "Upgrade")
        .body(axum::body::Body::empty())
        .unwrap()
        .into_response()
}

/// Handle upgraded Rapace connection
async fn handle_rapace_connection(upgraded: Upgraded, service: Arc<HindsightServiceImpl>) {
    tracing::info!("Handling Rapace connection over HTTP upgrade");

    // Upgraded is not Sync, but Rapace requires Sync for Transport.
    // Solution: Use DuplexStream as a bridge (it's Sync)
    let (mut client_stream, server_stream) = tokio::io::duplex(8192);

    // Spawn a task to bridge the Upgraded connection to DuplexStream
    tokio::spawn(async move {
        let mut upgraded = TokioIo::new(upgraded);
        if let Err(e) = tokio::io::copy_bidirectional(&mut upgraded, &mut client_stream).await {
            tracing::error!("HTTP upgrade bridge error: {}", e);
        }
    });

    // Use the Sync-safe DuplexStream with StreamTransport
    let transport = Arc::new(rapace::transport::StreamTransport::new(server_stream));

    // IMPORTANT: No tracer attached! (Prevents infinite loop)
    let session = Arc::new(RpcSession::new(transport));

    // Create dispatcher function
    session.set_dispatcher(move |_channel_id, method_id, payload| {
        let service_impl = service.as_ref().clone();
        Box::pin(async move {
            let server = HindsightServiceServer::new(service_impl);
            server.dispatch(method_id, &payload).await
        })
    });

    if let Err(e) = session.run().await {
        tracing::error!("Rapace HTTP upgrade session error: {}", e);
    }
}

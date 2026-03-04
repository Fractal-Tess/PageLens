//! Server mode functionality

use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
#[cfg(not(debug_assertions))]
use std::net::SocketAddr;

use pagelens_logging::{info, warn};

#[cfg(not(debug_assertions))]
use axum::http::header;
#[cfg(not(debug_assertions))]
use axum::http::{Method, StatusCode, Uri};
#[cfg(not(debug_assertions))]
use axum::response::{IntoResponse, Response};
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};

#[cfg(not(debug_assertions))]
static EMBEDDED_WEB_DIST: Dir<'_> = include_dir!("$OUT_DIR/pagelens-web");

pub fn default_server_db_path() -> PathBuf {
    PathBuf::from(".pagelens/pagelens.db")
}

pub fn open_browser_for_server(host: IpAddr, port: u16) {
    let open_host = if host.is_unspecified() {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        host
    };
    let url = format!("http://{}:{}/", open_host, port);
    info!(url = %url, "Attempting to open browser for server mode");
    if let Err(err) = webbrowser::open(&url) {
        warn!(url = %url, error = %err, "Failed to open browser automatically");
        eprintln!("Warning: failed to open browser automatically: {err}");
    }
}

pub async fn run_server(host: IpAddr, port: u16, db_path: PathBuf) -> Result<(), String> {
    info!(host = %host, port, db_path = %db_path.display(), "Running server mode");
    #[cfg(debug_assertions)]
    {
        let config = pagelens_api::ApiConfig {
            host,
            port,
            db_path,
        };
        return pagelens_api::serve(config).await;
    }

    #[cfg(not(debug_assertions))]
    {
        let service = pagelens_api::service_from_db_path(db_path)?;
        let api_router = pagelens_api::router(service);
        let app = api_router.fallback(serve_embedded_web);

        let addr = SocketAddr::new(host, port);
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| e.to_string())?;
        return axum::serve(listener, app).await.map_err(|e| e.to_string());
    }
}

#[cfg(not(debug_assertions))]
async fn serve_embedded_web(method: Method, uri: Uri) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed").into_response();
    }
    serve_embedded_asset(uri.path())
}

#[cfg(not(debug_assertions))]
fn serve_embedded_asset(path: &str) -> Response {
    let requested = {
        let value = path.trim_start_matches('/');
        if value.is_empty() {
            "index.html"
        } else {
            value
        }
    };

    if let Some(file) = EMBEDDED_WEB_DIST.get_file(requested) {
        return response_from_embed_file(file.contents(), requested);
    }

    let looks_like_asset = requested
        .rsplit('/')
        .next()
        .is_some_and(|segment| segment.contains('.'));
    if !looks_like_asset {
        if let Some(file) = EMBEDDED_WEB_DIST.get_file("index.html") {
            return response_from_embed_file(file.contents(), "index.html");
        }
    }

    (StatusCode::NOT_FOUND, "Not found").into_response()
}

#[cfg(not(debug_assertions))]
fn response_from_embed_file(contents: &'static [u8], path: &str) -> Response {
    let content_type = match path.rsplit('.').next().unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "wasm" => "application/wasm",
        "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };

    ([(header::CONTENT_TYPE, content_type)], contents).into_response()
}

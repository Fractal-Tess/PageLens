//! Server mode functionality

use std::net::{IpAddr, Ipv4Addr};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;

use pagelens_logging::{info, warn};

use axum::http::header;
use axum::http::HeaderMap;
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use include_dir::{include_dir, Dir};

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
    if let Err(err) = open_url(&url) {
        warn!(url = %url, error = %err, "Failed to open browser automatically");
        eprintln!("Warning: failed to open browser automatically: {err}");
    }
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .map_err(|err| err.to_string())
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("process exited with status {status}"))
                }
            })
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .status()
            .map_err(|err| err.to_string())
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("process exited with status {status}"))
                }
            })
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|err| err.to_string())
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("process exited with status {status}"))
                }
            })
    }
}

pub async fn run_server(host: IpAddr, port: u16, db_path: PathBuf) -> Result<(), String> {
    info!(host = %host, port, db_path = %db_path.display(), "Running server mode");
    let service = pagelens_api::service_from_db_path(db_path)?;
    let api_router = pagelens_api::router(service);
    let app = api_router.fallback(serve_embedded_web);

    let addr = SocketAddr::new(host, port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| e.to_string())?;
    axum::serve(listener, app).await.map_err(|e| e.to_string())
}

async fn serve_embedded_web(method: Method, uri: Uri, headers: HeaderMap) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed").into_response();
    }
    let accepts_html = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_none_or(|value| {
            value.contains("text/html") || value.contains("application/xhtml+xml") || value.contains("*/*")
        });

    serve_embedded_asset(uri.path(), accepts_html)
}

fn serve_embedded_asset(path: &str, allow_spa_fallback: bool) -> Response {
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
    let is_api_path = requested.starts_with("api/");

    if !looks_like_asset && !is_api_path && allow_spa_fallback {
        if let Some(file) = EMBEDDED_WEB_DIST.get_file("index.html") {
            return response_from_embed_file(file.contents(), "index.html");
        }
    }

    (StatusCode::NOT_FOUND, "Not found").into_response()
}

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

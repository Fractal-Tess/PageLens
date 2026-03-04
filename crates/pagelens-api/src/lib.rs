use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use pagelens_app::{AnalyseUrlInput, AppService, EditRunInput};
use pagelens_logging::{error, info, warn};
use serde::Deserialize;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::pin::Pin;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: IpAddr,
    pub port: u16,
    pub db_path: PathBuf,
}

#[derive(Clone)]
struct ApiState {
    service: AppService,
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn serve(config: ApiConfig) -> Result<(), String> {
    pagelens_logging::init("pagelens-api");
    let service = service_from_db_path(config.db_path)?;
    let app = router(service);

    let addr = SocketAddr::new(config.host, config.port);
    info!(address = %addr, "Starting API server");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| e.to_string())?;

    axum::serve(listener, app).await.map_err(|e| e.to_string())
}

pub fn service_from_db_path(db_path: PathBuf) -> Result<AppService, String> {
    AppService::new(db_path).map_err(|e| e.to_string())
}

pub fn router(service: AppService) -> Router {
    let assets_dir = service.assets_base_dir().to_path_buf();
    let state = ApiState { service };
    Router::new()
        .route("/health", get(health))
        .route("/api/runs/analyse", post(start_analyse))
        .route("/api/runs/{run_id}", get(get_run).patch(edit_run))
        .route("/api/runs/{run_id}/cancel", post(cancel_run))
        .route("/api/runs/{run_id}/pages", get(get_pages))
        .route("/api/runs/{run_id}/assets", get(get_run_assets))
        .route("/api/runs/{run_id}/events", get(run_events))
        .route("/api/history", get(list_history))
        .nest_service("/api/assets", ServeDir::new(assets_dir))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn start_analyse(
    State(state): State<ApiState>,
    Json(input): Json<AnalyseUrlInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!(url = %input.url, analysis_type = ?input.analysis_type, "Received analysis start request");
    let started = state
        .service
        .start_analyse(input)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(serde_json::json!(started)))
}

async fn get_run(
    State(state): State<ApiState>,
    Path(run_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let run = state.service.get_run(&run_id).map_err(ApiError::from_app)?;
    Ok(Json(serde_json::json!(run)))
}

async fn edit_run(
    State(state): State<ApiState>,
    Path(run_id): Path<String>,
    Json(input): Json<EditRunInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let run = state
        .service
        .edit_run(&run_id, input)
        .map_err(ApiError::from_app)?;
    Ok(Json(serde_json::json!(run)))
}

async fn cancel_run(
    State(state): State<ApiState>,
    Path(run_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!(run_id = %run_id, "Received cancel request");
    state
        .service
        .cancel_run(&run_id)
        .await
        .map_err(ApiError::from_app)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn get_pages(
    State(state): State<ApiState>,
    Path(run_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pages = state
        .service
        .list_run_pages(&run_id)
        .map_err(ApiError::from_app)?;
    Ok(Json(serde_json::json!(pages)))
}

async fn get_run_assets(
    State(state): State<ApiState>,
    Path(run_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let assets = state
        .service
        .list_run_assets(&run_id)
        .map_err(ApiError::from_app)?;
    Ok(Json(serde_json::json!(assets)))
}

async fn list_history(
    State(state): State<ApiState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);
    if limit < 0 || offset < 0 {
        return Err(ApiError::bad_request(
            "limit and offset must be non-negative",
        ));
    }

    let items = state
        .service
        .list_history(limit, offset)
        .map_err(ApiError::internal)?;
    Ok(Json(serde_json::json!(items)))
}

async fn run_events(
    State(state): State<ApiState>,
    Path(run_id): Path<String>,
) -> Result<Response, ApiError> {
    if let Some(rx) = state.service.subscribe(&run_id).await {
        info!(run_id = %run_id, "Opening SSE stream for active run");
        let stream = BroadcastStream::new(rx).filter_map(|result| match result {
            Ok(event) => serde_json::to_string(&event)
                .ok()
                .map(|data| Ok::<Event, Infallible>(Event::default().event(event.kind).data(data))),
            Err(_) => None,
        });
        return Ok(Sse::new(
            Box::pin(stream) as Pin<Box<dyn tokio_stream::Stream<Item = _> + Send>>
        )
        .keep_alive(KeepAlive::default())
        .into_response());
    }

    match state.service.terminal_event_for_run(&run_id) {
        Ok(Some(event)) => {
            info!(run_id = %run_id, event_kind = %event.kind, "Serving terminal SSE event");
            let data = serde_json::to_string(&event).map_err(ApiError::internal)?;
            let one = tokio_stream::once(Ok::<Event, Infallible>(
                Event::default().event(event.kind).data(data),
            ));
            Ok(
                Sse::new(Box::pin(one) as Pin<Box<dyn tokio_stream::Stream<Item = _> + Send>>)
                    .keep_alive(KeepAlive::default())
                    .into_response(),
            )
        }
        Ok(None) => {
            warn!(run_id = %run_id, "SSE requested for unknown run");
            Err(ApiError::not_found("Run not found"))
        }
        Err(err) => {
            error!(run_id = %run_id, error = %err, "Failed to resolve terminal SSE event");
            Err(ApiError::from_app(err))
        }
    }
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn internal<E: ToString>(err: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }

    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.to_string(),
        }
    }

    fn from_app(err: pagelens_app::Error) -> Self {
        if err.is_not_found() {
            Self::not_found(&err.to_string())
        } else if matches!(err, pagelens_app::Error::Message(_)) {
            Self::bad_request(&err.to_string())
        } else {
            Self::internal(err)
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({
                "error": self.message,
            })),
        )
            .into_response()
    }
}

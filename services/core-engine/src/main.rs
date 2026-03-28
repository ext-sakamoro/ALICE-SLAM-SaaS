use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Debug, Default, Serialize)]
struct Stats {
    maps_updated: u64,
    localizations: u64,
    loop_closures: u64,
    exports: u64,
    total_requests: u64,
}

type AppState = Arc<Mutex<Stats>>;

// --- request / response types ---

#[derive(Debug, Deserialize)]
struct MapRequest {
    session_id: String,
    scan: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct LocalizeRequest {
    session_id: String,
    scan: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct LoopClosureRequest {
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct ExportQuery {
    session_id: Option<String>,
    format: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize> {
    ok: bool,
    request_id: String,
    data: T,
}

fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        ok: true,
        request_id: Uuid::new_v4().to_string(),
        data,
    })
}

// --- handlers ---

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "alice-slam-saas-core" }))
}

async fn slam_map(
    State(state): State<AppState>,
    Json(req): Json<MapRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.maps_updated += 1;
    s.total_requests += 1;
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "session_id": req.session_id,
            "keyframes_total": s.maps_updated,
            "map_updated": true,
        })),
    )
}

async fn slam_localize(
    State(state): State<AppState>,
    Json(req): Json<LocalizeRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.localizations += 1;
    s.total_requests += 1;
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "session_id": req.session_id,
            "pose": { "x": 0.0, "y": 0.0, "z": 0.0, "qw": 1.0, "qx": 0.0, "qy": 0.0, "qz": 0.0 },
            "confidence": 0.97,
        })),
    )
}

async fn slam_loop_closure(
    State(state): State<AppState>,
    Json(req): Json<LoopClosureRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.loop_closures += 1;
    s.total_requests += 1;
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "session_id": req.session_id,
            "loop_detected": true,
            "graph_nodes": 42,
            "pose_error_m": 0.003,
        })),
    )
}

async fn slam_export(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<ExportQuery>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.exports += 1;
    s.total_requests += 1;
    let fmt = q.format.unwrap_or_else(|| "ply".to_string());
    let sid = q.session_id.unwrap_or_else(|| "default".to_string());
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "session_id": sid,
            "format": fmt,
            "point_count": 128_000,
            "download_url": format!("/exports/{}.{}", sid, fmt),
        })),
    )
}

async fn slam_stats(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let s = state.lock().unwrap();
    ok(serde_json::json!({
        "maps_updated": s.maps_updated,
        "localizations": s.localizations,
        "loop_closures": s.loop_closures,
        "exports": s.exports,
        "total_requests": s.total_requests,
    }))
}

// --- main ---

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state: AppState = Arc::new(Mutex::new(Stats::default()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/slam/map", post(slam_map))
        .route("/api/v1/slam/localize", post(slam_localize))
        .route("/api/v1/slam/loop-closure", post(slam_loop_closure))
        .route("/api/v1/slam/export", get(slam_export))
        .route("/api/v1/slam/stats", get(slam_stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8117".to_string())
        .parse()
        .unwrap_or(8117);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("alice-slam-saas-core listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

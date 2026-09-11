mod ocr;
mod schemes;

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use schemes::{match_schemes, SchemeMatch, UserProfile};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/match", post(match_handler))
        .route("/api/extract-document", post(extract_handler))
        .fallback_service(ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("scheme-setu listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn match_handler(Json(profile): Json<UserProfile>) -> impl IntoResponse {
    let matches: Vec<SchemeMatch> = match_schemes(&profile);
    Json(serde_json::json!({
        "matched_count": matches.len(),
        "schemes": matches,
    }))
}

/// Accepts a single-image multipart upload, extracts structured fields via
/// the vision model, and returns them. The image bytes exist only for the
/// lifetime of this request handler — see ocr.rs for the privacy rationale.
async fn extract_handler(mut multipart: Multipart) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let content_type = field.content_type().unwrap_or("image/jpeg").to_string();
        let Ok(data) = field.bytes().await else {
            continue;
        };
        match ocr::extract_fields_from_document(&data, &content_type).await {
            Ok(fields) => return (StatusCode::OK, Json(fields)).into_response(),
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({"error": e})),
                )
                    .into_response()
            }
        }
    }
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "no file field found"})),
    )
        .into_response()
}

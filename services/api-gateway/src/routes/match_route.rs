use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use labhsathi_core::schemes::{match_schemes, SchemeMatch, UserProfile};

use crate::catalog_client;
use crate::state::AppState;

pub async fn match_handler(State(state): State<AppState>, Json(profile): Json<UserProfile>) -> impl IntoResponse {
    let facts = match catalog_client::fetch_schemes(&state.http_client, &state.catalog_service_url).await {
        Ok(facts) => facts,
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch the scheme catalog");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "scheme catalog is temporarily unavailable, try again shortly" })),
            )
                .into_response();
        }
    };

    let matches: Vec<SchemeMatch> = match_schemes(&profile, &facts);
    Json(serde_json::json!({
        "matched_count": matches.len(),
        "schemes": matches,
    }))
    .into_response()
}

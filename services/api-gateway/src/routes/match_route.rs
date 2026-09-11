use axum::response::{IntoResponse, Json};
use labhsathi_core::schemes::{match_schemes, SchemeMatch, UserProfile};

pub async fn match_handler(Json(profile): Json<UserProfile>) -> impl IntoResponse {
    let matches: Vec<SchemeMatch> = match_schemes(&profile);
    Json(serde_json::json!({
        "matched_count": matches.len(),
        "schemes": matches,
    }))
}

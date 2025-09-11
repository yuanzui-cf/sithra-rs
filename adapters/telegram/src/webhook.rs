use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};

use crate::{state::AppState, types::Update};

pub async fn webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<Update>,
) -> impl IntoResponse {
    let header_secret = match headers.get("X-Telegram-Bot-Api-Secret-Token") {
        Some(secret) => secret.as_bytes(),
        None => b"",
    };

    if let Some(secret) = state.secret
        && header_secret != secret.as_bytes()
    {
        log::error!("Invalid secret token");
        return;
    }

    todo!("Emit event")
}

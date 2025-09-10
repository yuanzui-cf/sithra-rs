use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use serde_json::json;

use crate::{state::AppState, types::Update};

pub async fn webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<Update>,
) -> impl IntoResponse {
    log::info!("Headers: {headers:?}");
    log::info!("Request: {:?}", req);

    Json(json!("{}"))
}

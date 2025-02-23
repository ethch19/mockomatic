use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message, Utf8Bytes}},
    response::IntoResponse,
    routing::{Router, get},
    Extension,
};
use crate::http::AppState;
use crate::http::users::AccessClaims;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset, Time};

pub fn router() -> Router::<AppState> {
    Router::new()
        .route("/ws", get(websocket_handler))
}

#[derive(Serialize)]
struct NotifyMessage {
    event: String,
    mp3_data: String, // Base64-encoded MP3
    play_at: i64,     // Unix timestamp in milliseconds (UTC)
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, claims))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, claims: AccessClaims) {
    let mut rx = state.tx.subscribe();

    // Example: Broadcast an MP3 to play at 12:00 PM UTC today
    let today = OffsetDateTime::now_utc().date();
    let play_time = today.with_time(Time::from_hms(12, 0, 0).unwrap()).assume_utc();
    let play_at = play_time.unix_timestamp(); // Convert to milliseconds

    let message = NotifyMessage {
        event: "play_audio".to_string(),
        mp3_data: "base64_encoded_mp3_data_here".to_string(), // Replace with actual Base64 MP3
        play_at,
    };
    
}
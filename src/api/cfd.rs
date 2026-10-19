//! QORDY POS - Customer Facing Display (CFD) & WebSocket Broadcast API

use crate::api::AppState;
use crate::models::CfdState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures_util::StreamExt;
use std::sync::Arc;

pub async fn handle_ws_pos(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(socket, state))
}

async fn handle_ws_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.ws_tx.subscribe();

    // Bağlanır bağlanmaz güncel CFD durumunu müşteriye yolla
    let current_cfd = state.db.get_cfd_state();
    if let Ok(json) = serde_json::to_string(&serde_json::json!({
        "type": "CFD_UPDATE",
        "data": current_cfd
    })) {
        let _ = socket.send(Message::Text(json)).await;
    }

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            client_msg = socket.next() => {
                match client_msg {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(parsed) = serde_json::from_str::<CfdState>(&txt) {
                            let _ = state.db.update_cfd_state(&parsed);
                            let broadcast_msg = serde_json::json!({
                                "type": "CFD_UPDATE",
                                "data": parsed
                            }).to_string();
                            let _ = state.ws_tx.send(broadcast_msg);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

pub async fn handle_get_cfd_state(State(state): State<Arc<AppState>>) -> Json<CfdState> {
    let cfd = state.db.get_cfd_state();
    Json(cfd)
}

pub async fn handle_update_cfd_state(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CfdState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if let Err(e) = state.db.update_cfd_state(&req) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("CFD kaydedilemedi: {}", e),
        ));
    }

    // WebSocket üzerinden tüm 2. ekranlara anlık fırlat
    let msg = serde_json::json!({
        "type": "CFD_UPDATE",
        "data": req
    })
    .to_string();
    let _ = state.ws_tx.send(msg);

    Ok(Json(serde_json::json!({ "success": true })))
}

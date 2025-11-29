use std::sync::Arc;

use axum::{
    Json, Router,
    extract::FromRef,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::Level;

use crate::services::{
    data::DataServiceHandle,
    game::{GameService, GameServiceHandle},
    lobby::{LobbyService, LobbyServiceHandle},
    ws_connection::{WsConnectionService, WsConnectionServiceHandle},
};

pub mod game;
mod routes;
pub mod services;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error("Failed to read file at {0}")]
    FailedToReadFile(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::FailedToReadFile(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}

#[derive(Clone)]
pub struct AppState {
    data: DataServiceHandle,
    lobby: LobbyServiceHandle,
    game: GameServiceHandle,
    ws_connection: WsConnectionServiceHandle,
}

impl FromRef<AppState> for DataServiceHandle {
    fn from_ref(input: &AppState) -> Self {
        input.data.clone()
    }
}

impl FromRef<AppState> for LobbyServiceHandle {
    fn from_ref(input: &AppState) -> Self {
        input.lobby.clone()
    }
}

impl FromRef<AppState> for GameServiceHandle {
    fn from_ref(input: &AppState) -> Self {
        input.game.clone()
    }
}

impl FromRef<AppState> for WsConnectionServiceHandle {
    fn from_ref(input: &AppState) -> Self {
        input.ws_connection.clone()
    }
}

pub fn app(data_service: DataServiceHandle) -> Router {
    let cors_layer = CorsLayer::new()
        .allow_headers(Any)
        .allow_origin(Any)
        .allow_methods(Any);

    let ws_connection = Arc::new(Mutex::new(WsConnectionService::default()));

    let state = AppState {
        data: data_service.clone(),
        lobby: Arc::new(Mutex::new(LobbyService::default())),
        ws_connection: ws_connection.clone(),
        game: Arc::new(Mutex::new(GameService::new(
            data_service.clone(),
            ws_connection.clone(),
        ))),
    };

    Router::new()
        .merge(routes::routes(state))
        .layer(ServiceBuilder::new().layer(cors_layer))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}

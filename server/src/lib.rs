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
    data::DataService,
    lobby::{LobbyService, LobbyServiceHandle},
};

mod routes;
mod services;

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
    data: Arc<DataService>,
    lobby: LobbyServiceHandle,
}

impl FromRef<AppState> for Arc<DataService> {
    fn from_ref(input: &AppState) -> Self {
        input.data.clone()
    }
}

impl FromRef<AppState> for LobbyServiceHandle {
    fn from_ref(input: &AppState) -> Self {
        input.lobby.clone()
    }
}

pub fn app() -> Router {
    let cors_layer = CorsLayer::new()
        .allow_headers(Any)
        .allow_origin(Any)
        .allow_methods(Any);

    let state = AppState {
        data: Arc::new(DataService),
        lobby: Arc::new(Mutex::new(LobbyService::default())),
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

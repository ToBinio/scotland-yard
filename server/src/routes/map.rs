use axum::{Router, extract::State, routing::get};

use crate::{
    AppState,
    services::data::{Connection, DataService, Round, Station},
};

use std::sync::Arc;

use axum::Json;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/stations", get(get_all_stations))
        .route("/connections", get(get_all_connections))
        .route("/rounds", get(get_all_rounds))
        .with_state(state)
}

async fn get_all_stations(State(data_service): State<Arc<DataService>>) -> Json<Vec<Station>> {
    Json(data_service.get_all_stations())
}

async fn get_all_connections(
    State(data_service): State<Arc<DataService>>,
) -> Json<Vec<Connection>> {
    Json(data_service.get_all_connections())
}

async fn get_all_rounds(State(data_service): State<Arc<DataService>>) -> Json<Vec<Round>> {
    Json(data_service.get_all_rounds())
}

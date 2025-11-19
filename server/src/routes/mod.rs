use axum::Router;

use crate::AppState;

mod game;
mod map;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest("/game", game::routes(state.clone()))
        .nest("/map", map::routes(state.clone()))
}

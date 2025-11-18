use axum::Router;

use crate::AppState;

mod map;

pub fn routes(state: AppState) -> Router {
    Router::new().nest("/map", map::routes(state.clone()))
}

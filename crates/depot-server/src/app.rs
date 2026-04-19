use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Build the axum application with all middleware and adapter routes.
pub fn build_app(state: AppState) -> Router {
    Router::new()
        // Protocol adapter routes will be nested here:
        // .nest("/pypi", pypi::router(state.clone()))
        // .nest("/npm", npm::router(state.clone()))
        // .nest("/cargo", cargo::router(state.clone()))
        // .nest("/hex", hex::router(state.clone()))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

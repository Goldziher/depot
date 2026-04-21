use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Build the axum application with all middleware and adapter routes.
pub fn build_app(state: AppState) -> Router {
    let mut app = Router::new();

    #[cfg(feature = "pypi")]
    {
        app = app.nest("/pypi", depot_adapters::pypi::router());
    }

    #[cfg(feature = "npm")]
    {
        app = app.nest("/npm", depot_adapters::npm::router());
    }

    #[cfg(feature = "cargo-registry")]
    {
        app = app.nest("/cargo", depot_adapters::cargo::router());
    }

    #[cfg(feature = "hex")]
    {
        app = app.nest("/hex", depot_adapters::hex::router());
    }

    app.layer(CompressionLayer::new())
        // TODO: replace CorsLayer::permissive() with a restrictive policy before production use
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

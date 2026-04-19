use std::sync::Arc;

use depot_core::config::Config;

/// Shared application state, passed to all handlers via axum's State extractor.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    // TODO: add storage, upstream clients, policy engine
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

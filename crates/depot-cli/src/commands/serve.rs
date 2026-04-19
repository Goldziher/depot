use depot_core::config::Config;
use depot_server::app::build_app;
use depot_server::state::AppState;

pub fn run(config: Config) {
    let state = AppState::new(config.clone());
    let app = build_app(state);

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(&config.server.bind)
            .await
            .unwrap_or_else(|e| {
                eprintln!("error: failed to bind to {}: {e}", config.server.bind);
                std::process::exit(1);
            });

        tracing::info!("depot listening on {}", config.server.bind);
        axum::serve(listener, app).await.expect("server error");
    });
}

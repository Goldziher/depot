use std::sync::Arc;

use ahash::AHashMap;
use depot_core::config::Config;
use depot_core::package::Ecosystem;
use depot_core::ports::UpstreamClient;
use depot_server::app::build_app;
use depot_server::state::AppState;
use depot_service::CachingPackageService;
use depot_storage::OpenDalStorage;

pub fn run(config: Config) {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let storage = build_storage(&config);
        let mut clients: AHashMap<Ecosystem, Arc<dyn UpstreamClient>> = AHashMap::new();

        #[cfg(feature = "full")]
        let (pypi_upstream, cargo_upstream, npm_upstream, hex_upstream) =
            register_upstream_clients(&config, &mut clients);

        let service =
            CachingPackageService::new(Arc::new(storage), clients, config.policies.clone());

        let state = AppState::new(
            config.clone(),
            Arc::new(service),
            #[cfg(feature = "full")]
            pypi_upstream,
            #[cfg(feature = "full")]
            cargo_upstream,
            #[cfg(feature = "full")]
            npm_upstream,
            #[cfg(feature = "full")]
            hex_upstream,
        );
        let app = build_app(state);

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

fn build_storage(config: &Config) -> OpenDalStorage {
    match config.storage.backend.as_str() {
        #[cfg(feature = "backend-fs")]
        "fs" => {
            let path = config
                .storage
                .path
                .as_deref()
                .unwrap_or_else(|| std::path::Path::new("./depot-data"));
            OpenDalStorage::filesystem(path).unwrap_or_else(|e| {
                eprintln!("error: failed to create fs storage: {e}");
                std::process::exit(1);
            })
        }
        #[cfg(feature = "backend-memory")]
        "memory" => OpenDalStorage::memory().unwrap_or_else(|e| {
            eprintln!("error: failed to create memory storage: {e}");
            std::process::exit(1);
        }),
        other => {
            eprintln!("error: unsupported storage backend: {other}");
            eprintln!("supported backends: fs, memory (s3 and gcs require feature flags)");
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "full")]
fn register_upstream_clients(
    config: &Config,
    clients: &mut AHashMap<Ecosystem, Arc<dyn UpstreamClient>>,
) -> (
    Arc<depot_adapters::pypi::upstream::PypiUpstreamClient>,
    Arc<depot_adapters::cargo::upstream::CargoUpstreamClient>,
    Arc<depot_adapters::npm::upstream::NpmUpstreamClient>,
    Arc<depot_adapters::hex::upstream::HexUpstreamClient>,
) {
    let pypi_upstream = if let Some(pypi_config) = config.upstream.get("pypi")
        && pypi_config.enabled
    {
        let client = Arc::new(depot_adapters::pypi::upstream::PypiUpstreamClient::new(
            pypi_config.url.clone(),
        ));
        clients.insert(Ecosystem::PyPI, client.clone());
        tracing::info!("PyPI upstream enabled: {}", pypi_config.url);
        client
    } else {
        Arc::new(depot_adapters::pypi::upstream::PypiUpstreamClient::new(
            "https://pypi.org".to_string(),
        ))
    };

    let npm_upstream = if let Some(npm_config) = config.upstream.get("npm")
        && npm_config.enabled
    {
        let client = Arc::new(depot_adapters::npm::upstream::NpmUpstreamClient::new(
            npm_config.url.clone(),
        ));
        clients.insert(Ecosystem::Npm, client.clone());
        tracing::info!("npm upstream enabled: {}", npm_config.url);
        client
    } else {
        Arc::new(depot_adapters::npm::upstream::NpmUpstreamClient::new(
            "https://registry.npmjs.org".to_string(),
        ))
    };

    let cargo_upstream = if let Some(cargo_config) = config.upstream.get("cargo")
        && cargo_config.enabled
    {
        let client = Arc::new(depot_adapters::cargo::upstream::CargoUpstreamClient::new(
            cargo_config.url.clone(),
            "https://static.crates.io/crates".to_string(),
        ));
        clients.insert(Ecosystem::Cargo, client.clone());
        tracing::info!("Cargo upstream enabled: {}", cargo_config.url);
        client
    } else {
        Arc::new(depot_adapters::cargo::upstream::CargoUpstreamClient::new(
            "https://index.crates.io".to_string(),
            "https://static.crates.io/crates".to_string(),
        ))
    };

    let hex_upstream = if let Some(hex_config) = config.upstream.get("hex")
        && hex_config.enabled
    {
        let client = Arc::new(depot_adapters::hex::upstream::HexUpstreamClient::new(
            hex_config.url.clone(),
            "https://repo.hex.pm".to_string(),
        ));
        clients.insert(Ecosystem::Hex, client.clone());
        tracing::info!("Hex upstream enabled: {}", hex_config.url);
        client
    } else {
        Arc::new(depot_adapters::hex::upstream::HexUpstreamClient::new(
            "https://hex.pm".to_string(),
            "https://repo.hex.pm".to_string(),
        ))
    };

    (pypi_upstream, cargo_upstream, npm_upstream, hex_upstream)
}

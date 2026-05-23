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
        config.validate_mvp().unwrap_or_else(|e| {
            eprintln!("error: {e}");
            std::process::exit(1);
        });

        let storage = build_storage(&config);
        let mut clients: AHashMap<Ecosystem, Arc<dyn UpstreamClient>> = AHashMap::new();

        #[cfg(feature = "pypi")]
        let pypi_upstream = register_pypi_upstream(&config, &mut clients);
        #[cfg(feature = "cargo-registry")]
        let cargo_upstream = register_cargo_upstream(&config, &mut clients);
        #[cfg(feature = "npm")]
        let npm_upstream = register_npm_upstream(&config, &mut clients);
        #[cfg(feature = "hex")]
        let hex_upstream = register_hex_upstream(&config, &mut clients);

        let service =
            CachingPackageService::new(Arc::new(storage), clients, config.policies.clone());

        let state = AppState::new(
            config.clone(),
            Arc::new(service),
            #[cfg(feature = "pypi")]
            pypi_upstream,
            #[cfg(feature = "cargo-registry")]
            cargo_upstream,
            #[cfg(feature = "npm")]
            npm_upstream,
            #[cfg(feature = "hex")]
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
    OpenDalStorage::from_config(&config.storage).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    })
}

#[cfg(feature = "pypi")]
fn register_pypi_upstream(
    config: &Config,
    clients: &mut AHashMap<Ecosystem, Arc<dyn UpstreamClient>>,
) -> Arc<depot_adapters::pypi::upstream::PypiUpstreamClient> {
    if let Some(pypi_config) = config.upstream.get("pypi")
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
    }
}

#[cfg(feature = "npm")]
fn register_npm_upstream(
    config: &Config,
    clients: &mut AHashMap<Ecosystem, Arc<dyn UpstreamClient>>,
) -> Arc<depot_adapters::npm::upstream::NpmUpstreamClient> {
    if let Some(npm_config) = config.upstream.get("npm")
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
    }
}

#[cfg(feature = "cargo-registry")]
fn register_cargo_upstream(
    config: &Config,
    clients: &mut AHashMap<Ecosystem, Arc<dyn UpstreamClient>>,
) -> Arc<depot_adapters::cargo::upstream::CargoUpstreamClient> {
    if let Some(cargo_config) = config.upstream.get("cargo")
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
    }
}

#[cfg(feature = "hex")]
fn register_hex_upstream(
    config: &Config,
    clients: &mut AHashMap<Ecosystem, Arc<dyn UpstreamClient>>,
) -> Arc<depot_adapters::hex::upstream::HexUpstreamClient> {
    if let Some(hex_config) = config.upstream.get("hex")
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
    }
}

use std::sync::Arc;

use depot_core::config::Config;
use depot_core::ports::PackageService;

/// Shared application state, passed to all handlers via axum's State extractor.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub package_service: Arc<dyn PackageService>,
    #[cfg(feature = "pypi")]
    pub pypi_upstream: Arc<depot_adapters::pypi::upstream::PypiUpstreamClient>,
    #[cfg(feature = "cargo-registry")]
    pub cargo_upstream: Arc<depot_adapters::cargo::upstream::CargoUpstreamClient>,
    #[cfg(feature = "npm")]
    pub npm_upstream: Arc<depot_adapters::npm::upstream::NpmUpstreamClient>,
    #[cfg(feature = "hex")]
    pub hex_upstream: Arc<depot_adapters::hex::upstream::HexUpstreamClient>,
}

impl AppState {
    pub fn new(
        config: Config,
        package_service: Arc<dyn PackageService>,
        #[cfg(feature = "pypi")] pypi_upstream: Arc<
            depot_adapters::pypi::upstream::PypiUpstreamClient,
        >,
        #[cfg(feature = "cargo-registry")] cargo_upstream: Arc<
            depot_adapters::cargo::upstream::CargoUpstreamClient,
        >,
        #[cfg(feature = "npm")] npm_upstream: Arc<depot_adapters::npm::upstream::NpmUpstreamClient>,
        #[cfg(feature = "hex")] hex_upstream: Arc<depot_adapters::hex::upstream::HexUpstreamClient>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            package_service,
            #[cfg(feature = "pypi")]
            pypi_upstream,
            #[cfg(feature = "cargo-registry")]
            cargo_upstream,
            #[cfg(feature = "npm")]
            npm_upstream,
            #[cfg(feature = "hex")]
            hex_upstream,
        }
    }
}

#[cfg(feature = "pypi")]
impl depot_adapters::pypi::HasPypiState for AppState {
    fn package_service(&self) -> &Arc<dyn PackageService> {
        &self.package_service
    }

    fn pypi_upstream(&self) -> &Arc<depot_adapters::pypi::upstream::PypiUpstreamClient> {
        &self.pypi_upstream
    }
}

#[cfg(feature = "npm")]
impl depot_adapters::npm::HasNpmState for AppState {
    fn package_service(&self) -> &Arc<dyn PackageService> {
        &self.package_service
    }

    fn npm_upstream(&self) -> &Arc<depot_adapters::npm::upstream::NpmUpstreamClient> {
        &self.npm_upstream
    }
}

#[cfg(feature = "cargo-registry")]
impl depot_adapters::cargo::HasCargoState for AppState {
    fn package_service(&self) -> &Arc<dyn PackageService> {
        &self.package_service
    }

    fn cargo_upstream(&self) -> &Arc<depot_adapters::cargo::upstream::CargoUpstreamClient> {
        &self.cargo_upstream
    }
}

#[cfg(feature = "hex")]
impl depot_adapters::hex::HasHexState for AppState {
    fn package_service(&self) -> &Arc<dyn PackageService> {
        &self.package_service
    }

    fn hex_upstream(&self) -> &Arc<depot_adapters::hex::upstream::HexUpstreamClient> {
        &self.hex_upstream
    }
}

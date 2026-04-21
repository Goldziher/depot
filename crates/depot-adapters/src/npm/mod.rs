//! npm registry protocol adapter.
//!
//! Provides an axum router that translates npm registry API requests into
//! `PackageService` trait calls, and an upstream client for fetching
//! packages from registry.npmjs.org or any npm-compatible registry.

pub mod models;
pub mod upstream;

use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use depot_core::error::DepotError;
use depot_core::package::{ArtifactId, Ecosystem, PackageName};
use depot_core::ports::PackageService;

use self::upstream::NpmUpstreamClient;

/// Trait for extracting npm-specific state from application state.
///
/// The server crate implements this on its `AppState` to bridge the adapter
/// to the service layer and upstream client without creating a circular dependency.
pub trait HasNpmState: Clone + Send + Sync + 'static {
    fn package_service(&self) -> &Arc<dyn PackageService>;
    fn npm_upstream(&self) -> &Arc<NpmUpstreamClient>;
}

/// Build the npm adapter router.
///
/// Mount this under `/npm` in the top-level application router.
pub fn router<S: HasNpmState>() -> Router<S> {
    Router::new()
        .route("/{package}", get(package_metadata::<S>))
        .route("/@{scope}/{name}", get(scoped_package_metadata::<S>))
        .route("/{package}/-/{filename}", get(download_tarball::<S>))
        .route(
            "/@{scope}/{name}/-/{filename}",
            get(download_scoped_tarball::<S>),
        )
}

/// GET /{package} -- return packument JSON for an unscoped package.
async fn package_metadata<S: HasNpmState>(
    State(state): State<S>,
    headers: HeaderMap,
    Path(package): Path<String>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let name = PackageName::new(package);
    let host = extract_host(&headers);
    serve_packument(state, &name, &host).await
}

/// GET /@{scope}/{name} -- return packument JSON for a scoped package.
async fn scoped_package_metadata<S: HasNpmState>(
    State(state): State<S>,
    headers: HeaderMap,
    Path((scope, name)): Path<(String, String)>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let full_name = PackageName::new(format!("@{scope}/{name}"));
    let host = extract_host(&headers);
    serve_packument(state, &full_name, &host).await
}

fn extract_host(headers: &HeaderMap) -> String {
    headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost")
        .to_string()
}

/// Shared logic for serving packument responses.
///
/// Triggers the caching lifecycle through `PackageService`, then serves
/// the full upstream packument (preserving dependencies, scripts, etc.)
/// with tarball URLs rewritten to point through this depot instance.
///
/// Also pre-fetches all transitive dependencies so they are warm in cache
/// when the client requests them.
async fn serve_packument<S: HasNpmState>(
    state: S,
    name: &PackageName,
    host: &str,
) -> Result<axum::response::Response, (StatusCode, String)> {
    // Trigger caching lifecycle through PackageService
    let _versions = state
        .package_service()
        .list_versions(Ecosystem::Npm, name)
        .await
        .map_err(|err| map_error(&err))?;

    // Get full packument from upstream cache (preserves deps, scripts, etc.)
    let mut packument = state
        .npm_upstream()
        .get_cached_packument(name)
        .await
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "packument not cached after list_versions".to_string(),
            )
        })?;

    // Pre-fetch all dependencies concurrently so they're warm when the client asks
    prefetch_dependencies(&state, &packument).await;

    // Rewrite tarball URLs to point through depot
    let base_url = format!("http://{host}");
    models::rewrite_packument_tarball_urls(&mut packument, &base_url);

    let body = serde_json::to_string(&packument)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(([(header::CONTENT_TYPE, "application/json")], body).into_response())
}

/// Pre-fetch the full transitive dependency tree using BFS.
///
/// Discovers dependency names from each packument's `dependencies` field,
/// fetches them through `PackageService` (which triggers caching), and
/// continues until all reachable deps are fetched or max depth is reached.
async fn prefetch_dependencies<S: HasNpmState>(state: &S, packument: &serde_json::Value) {
    use ahash::AHashSet;

    const MAX_DEPTH: usize = 10;

    let initial_deps = models::extract_dependency_names(packument);
    if initial_deps.is_empty() {
        return;
    }

    let pkg_name = packument["name"].as_str().unwrap_or("unknown");
    tracing::debug!(
        package = %pkg_name,
        dep_count = initial_deps.len(),
        "pre-fetching npm dependency tree"
    );

    let mut visited: AHashSet<String> = AHashSet::new();
    // Add the root package itself to visited
    visited.insert(pkg_name.to_string());

    let mut current_level: Vec<String> = initial_deps.into_iter().collect();

    for depth in 0..MAX_DEPTH {
        if current_level.is_empty() {
            break;
        }

        // Filter out already-visited deps
        let to_fetch: Vec<String> = current_level
            .into_iter()
            .filter(|name| visited.insert(name.clone()))
            .collect();

        if to_fetch.is_empty() {
            break;
        }

        tracing::debug!(depth, count = to_fetch.len(), "fetching dependency level");

        // Fetch all deps at this level concurrently
        let mut tasks = tokio::task::JoinSet::new();
        for dep_name in to_fetch {
            let service = state.package_service().clone();
            let upstream = state.npm_upstream().clone();
            tasks.spawn(async move {
                let pkg_name = PackageName::new(&dep_name);
                if let Err(err) = service.list_versions(Ecosystem::Npm, &pkg_name).await {
                    tracing::warn!(dep = %dep_name, %err, "failed to pre-fetch npm dependency");
                    return (dep_name, Vec::new());
                }
                // Extract this dep's own dependencies for the next level
                let next_deps =
                    if let Some(dep_packument) = upstream.get_cached_packument(&pkg_name).await {
                        models::extract_dependency_names(&dep_packument)
                            .into_iter()
                            .collect()
                    } else {
                        Vec::new()
                    };
                (dep_name, next_deps)
            });
        }

        // Collect next level's deps from all results
        let mut next_level = Vec::new();
        while let Some(result) = tasks.join_next().await {
            if let Ok((_dep_name, child_deps)) = result {
                next_level.extend(child_deps);
            }
        }

        current_level = next_level;
    }
}

/// GET /{package}/-/{filename} -- download an unscoped package tarball.
async fn download_tarball<S: HasNpmState>(
    State(state): State<S>,
    Path((package, filename)): Path<(String, String)>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let name = PackageName::new(package);
    serve_tarball(state, &name, &filename).await
}

/// GET /@{scope}/{name}/-/{filename} -- download a scoped package tarball.
async fn download_scoped_tarball<S: HasNpmState>(
    State(state): State<S>,
    Path((scope, name, filename)): Path<(String, String, String)>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let full_name = PackageName::new(format!("@{scope}/{name}"));
    serve_tarball(state, &full_name, &filename).await
}

/// Shared logic for serving tarball downloads.
async fn serve_tarball<S: HasNpmState>(
    state: S,
    name: &PackageName,
    filename: &str,
) -> Result<axum::response::Response, (StatusCode, String)> {
    // Extract version from filename: {name}-{version}.tgz
    let version = extract_version_from_filename(name.as_str(), filename).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "invalid filename format".to_string(),
        )
    })?;

    let artifact_id = ArtifactId {
        ecosystem: Ecosystem::Npm,
        name: name.clone(),
        version,
        filename: filename.to_string(),
    };

    let data = state
        .package_service()
        .get_artifact(&artifact_id)
        .await
        .map_err(|err| map_error(&err))?;

    let disposition = format!("attachment; filename=\"{filename}\"");
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        data,
    )
        .into_response())
}

/// Extract version string from an npm tarball filename.
///
/// Expected format: `{name}-{version}.tgz` where name may contain `@scope/`.
fn extract_version_from_filename(name: &str, filename: &str) -> Option<String> {
    let stripped = filename.strip_suffix(".tgz")?;
    let prefix = format!("{name}-");
    let version = stripped.strip_prefix(&prefix)?;
    if version.is_empty() {
        return None;
    }
    Some(version.to_string())
}

/// Map `DepotError` variants to appropriate HTTP status codes.
fn map_error(err: &DepotError) -> (StatusCode, String) {
    match err {
        DepotError::PackageNotFound { .. }
        | DepotError::VersionNotFound { .. }
        | DepotError::ArtifactNotFound(_) => (StatusCode::NOT_FOUND, err.to_string()),
        DepotError::PolicyViolation(_) => (StatusCode::FORBIDDEN, err.to_string()),
        DepotError::Upstream(_) => (StatusCode::BAD_GATEWAY, err.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_version_from_simple_filename() {
        assert_eq!(
            extract_version_from_filename("is-odd", "is-odd-3.0.1.tgz"),
            Some("3.0.1".to_string())
        );
    }

    #[test]
    fn should_extract_version_from_scoped_filename() {
        assert_eq!(
            extract_version_from_filename("@scope/pkg", "@scope/pkg-1.2.3.tgz"),
            Some("1.2.3".to_string())
        );
    }

    #[test]
    fn should_return_none_for_invalid_filename() {
        assert_eq!(
            extract_version_from_filename("is-odd", "not-a-match.tgz"),
            None
        );
        assert_eq!(extract_version_from_filename("is-odd", "is-odd-.tgz"), None);
        assert_eq!(
            extract_version_from_filename("is-odd", "is-odd-1.0.0.tar.gz"),
            None
        );
    }
}

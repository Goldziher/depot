use serde_json::Value;

/// Resolve the workspace root from CARGO_MANIFEST_DIR (depot-core -> workspace).
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("could not resolve workspace root")
        .to_path_buf()
}

fn load_schema(relative_path: &str) -> Value {
    let path = workspace_root().join(relative_path);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read schema {}: {e}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse schema {}: {e}", path.display()))
}

fn validate(schema_value: &Value, instance: &Value) -> std::result::Result<(), String> {
    let validator =
        jsonschema::validator_for(schema_value).map_err(|e| format!("invalid schema: {e}"))?;
    let errors: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| e.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[test]
fn should_validate_pypi_sample_against_schema() {
    let schema = load_schema("schemas/registries/pypi.schema.json");
    let sample: Value = serde_json::from_str(
        r#"{
            "meta": { "api-version": "1.0" },
            "name": "requests",
            "versions": ["2.32.0"],
            "files": [{
                "filename": "requests-2.32.0.tar.gz",
                "url": "https://files.pythonhosted.org/packages/requests-2.32.0.tar.gz",
                "hashes": { "sha256": "abc123" },
                "requires-python": ">=3.8",
                "yanked": false,
                "size": 131200
            }]
        }"#,
    )
    .unwrap();

    validate(&schema, &sample).expect("PyPI sample should validate against schema");
}

#[test]
fn should_reject_pypi_sample_missing_required_fields() {
    let schema = load_schema("schemas/registries/pypi.schema.json");
    let invalid: Value = serde_json::from_str(r#"{ "name": "oops" }"#).unwrap();

    assert!(
        validate(&schema, &invalid).is_err(),
        "missing 'meta' and 'files' should fail validation"
    );
}

#[test]
fn should_validate_npm_sample_against_schema() {
    let schema = load_schema("schemas/registries/npm.schema.json");
    let sample: Value = serde_json::from_str(
        r#"{
            "name": "express",
            "dist-tags": { "latest": "4.21.0" },
            "versions": {
                "4.21.0": {
                    "name": "express",
                    "version": "4.21.0",
                    "dist": {
                        "tarball": "https://registry.npmjs.org/express/-/express-4.21.0.tgz",
                        "shasum": "d57cb706d49623d4ac27833f1cbc466b668eb915"
                    }
                }
            }
        }"#,
    )
    .unwrap();

    validate(&schema, &sample).expect("npm sample should validate against schema");
}

#[test]
fn should_validate_cargo_sample_against_schema() {
    let schema = load_schema("schemas/registries/cargo.schema.json");
    let sample: Value = serde_json::from_str(
        r#"{
            "name": "serde",
            "vers": "1.0.210",
            "deps": [],
            "cksum": "abc123",
            "features": {},
            "yanked": false
        }"#,
    )
    .unwrap();

    validate(&schema, &sample).expect("Cargo sample should validate against schema");
}

#[test]
fn should_reject_cargo_sample_missing_cksum() {
    let schema = load_schema("schemas/registries/cargo.schema.json");
    let invalid: Value = serde_json::from_str(
        r#"{
            "name": "serde",
            "vers": "1.0.210",
            "deps": [],
            "features": {},
            "yanked": false
        }"#,
    )
    .unwrap();

    assert!(
        validate(&schema, &invalid).is_err(),
        "missing 'cksum' should fail validation"
    );
}

#[test]
fn should_validate_hex_sample_against_schema() {
    let schema = load_schema("schemas/registries/hex.schema.json");
    let sample: Value = serde_json::from_str(
        r#"{
            "name": "phoenix",
            "releases": [
                {
                    "version": "1.7.14",
                    "url": "https://hex.pm/api/packages/phoenix/releases/1.7.14",
                    "has_docs": true
                }
            ]
        }"#,
    )
    .unwrap();

    validate(&schema, &sample).expect("Hex sample should validate against schema");
}

#[test]
fn should_validate_config_sample_against_schema() {
    let schema = load_schema("schemas/depot/config.schema.json");
    let sample: Value = serde_json::from_str(
        r#"{
            "server": { "bind": "0.0.0.0:8080" },
            "storage": { "backend": "fs" }
        }"#,
    )
    .unwrap();

    validate(&schema, &sample).expect("config sample should validate against schema");
}

#[test]
fn should_validate_lockfile_sample_against_schema() {
    let schema = load_schema("schemas/depot/lockfile.schema.json");
    let sample: Value = serde_json::from_str(
        r#"{
            "metadata": {
                "schema_version": 1,
                "generated_at": "2024-01-01T00:00:00Z",
                "depot_version": "0.1.0"
            },
            "packages": []
        }"#,
    )
    .unwrap();

    validate(&schema, &sample).expect("lockfile sample should validate against schema");
}

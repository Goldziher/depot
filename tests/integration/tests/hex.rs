use tokio::process::Command;

use depot_integration_tests::TestServer;

#[tokio::test]
#[ignore] // requires network
async fn hex_package_metadata_returns_json() {
    let server = TestServer::start().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/hex/api/packages/jason", server.base_url()))
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("invalid JSON response");
    assert_eq!(body["name"], "jason");
    assert!(
        body["releases"].is_array(),
        "expected releases array in response"
    );
    assert!(
        !body["releases"].as_array().unwrap().is_empty(),
        "expected at least one release"
    );

    server.shutdown();
}

#[tokio::test]
#[ignore] // requires network
async fn hex_tarball_download() {
    let server = TestServer::start().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/hex/tarballs/jason-1.4.1.tar",
            server.base_url()
        ))
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status(), 200);

    let bytes = response.bytes().await.expect("failed to read body");
    assert!(!bytes.is_empty(), "expected non-empty tarball body");

    server.shutdown();
}

#[tokio::test]
#[ignore] // requires network
async fn hex_nonexistent_package_returns_404() {
    let server = TestServer::start().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/hex/api/packages/this-does-not-exist-depot-test",
            server.base_url()
        ))
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status(), 404);

    server.shutdown();
}

#[tokio::test]
#[ignore] // requires network
async fn hex_package_has_license_info() {
    let server = TestServer::start().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/hex/api/packages/jason", server.base_url()))
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("invalid JSON response");
    let meta = &body["meta"];
    assert!(
        meta["licenses"].is_array(),
        "expected meta.licenses array in response"
    );
    assert!(
        !meta["licenses"].as_array().unwrap().is_empty(),
        "expected at least one license"
    );

    server.shutdown();
}

#[tokio::test]
#[ignore] // requires network
async fn hex_cached_on_second_request() {
    let server = TestServer::start().await;

    let client = reqwest::Client::new();

    let response1 = client
        .get(format!("{}/hex/api/packages/jason", server.base_url()))
        .send()
        .await
        .expect("first request failed");
    assert_eq!(response1.status(), 200);

    let response2 = client
        .get(format!("{}/hex/api/packages/jason", server.base_url()))
        .send()
        .await
        .expect("second request failed");
    assert_eq!(response2.status(), 200);

    server.shutdown();
}

#[tokio::test]
#[ignore] // requires network + mix
async fn mix_hex_package_fetch() {
    let server = TestServer::start().await;
    let hex_mirror = format!("{}/hex", server.base_url());

    let tmp = tempfile::tempdir().expect("tempdir");
    let output_path = tmp.path().join("jason-1.4.1.tar");

    let output = Command::new("mix")
        .args([
            "hex.package",
            "fetch",
            "jason",
            "1.4.1",
            "--output",
            &output_path.to_string_lossy(),
        ])
        .env("HEX_MIRROR", &hex_mirror)
        .output()
        .await
        .expect("failed to run mix");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "mix hex.package fetch failed.\nstdout: {stdout}\nstderr: {stderr}"
    );

    assert!(
        output_path.exists(),
        "tarball not written to {output_path:?}"
    );
    let size = std::fs::metadata(&output_path).unwrap().len();
    assert!(size > 0, "tarball is empty");

    server.shutdown();
}

#[tokio::test]
#[ignore] // requires network + mix
async fn mix_hex_package_fetch_cached() {
    let server = TestServer::start().await;
    let hex_mirror = format!("{}/hex", server.base_url());

    let tmp1 = tempfile::tempdir().expect("tempdir");
    let tmp2 = tempfile::tempdir().expect("tempdir");

    // First fetch
    let out1 = Command::new("mix")
        .args([
            "hex.package",
            "fetch",
            "jason",
            "1.4.1",
            "--output",
            &tmp1.path().join("jason.tar").to_string_lossy(),
        ])
        .env("HEX_MIRROR", &hex_mirror)
        .output()
        .await
        .expect("failed to run mix");
    assert!(out1.status.success(), "first mix fetch failed");

    // Second fetch — hits depot cache
    let out2 = Command::new("mix")
        .args([
            "hex.package",
            "fetch",
            "jason",
            "1.4.1",
            "--output",
            &tmp2.path().join("jason.tar").to_string_lossy(),
        ])
        .env("HEX_MIRROR", &hex_mirror)
        .output()
        .await
        .expect("failed to run mix");
    assert!(out2.status.success(), "second mix fetch (cached) failed");

    assert!(tmp2.path().join("jason.tar").exists());

    server.shutdown();
}

#[tokio::test]
#[ignore] // requires network + mix
async fn mix_hex_package_fetch_nonexistent_fails() {
    let server = TestServer::start().await;
    let hex_mirror = format!("{}/hex", server.base_url());

    let tmp = tempfile::tempdir().expect("tempdir");

    let output = Command::new("mix")
        .args([
            "hex.package",
            "fetch",
            "this-package-does-not-exist-depot-test",
            "0.0.1",
            "--output",
            &tmp.path().join("out.tar").to_string_lossy(),
        ])
        .env("HEX_MIRROR", &hex_mirror)
        .output()
        .await
        .expect("failed to run mix");

    assert!(
        !output.status.success(),
        "mix should fail for nonexistent package"
    );

    server.shutdown();
}

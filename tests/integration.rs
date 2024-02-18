//
// tests/integration.rs -- basic API end-to-end integration testing
//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the pcgtoolssoftware project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT

use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

const DEF_START_WAIT: u64 = 4;
const T_VALUE: &'static str = "helloworld";

// A utility function to prepare the environment before starting the server.
fn prepare_environment() {
    // Use CARGO_MANIFEST_DIR to get the path to the source directory
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Construct the source path for the configuration file
    let config_src_path = Path::new(&cargo_dir).join("example-cfg-kvapp.json");
    // Define the destination path for the configuration file
    let config_dest_path = Path::new("cfg-kvapp.json");

    // Copy the configuration file to the current directory.
    fs::copy(config_src_path, config_dest_path).expect("Failed to copy configuration file");

    // Create the db.kv directory if it does not exist.
    let db_dir = Path::new("db.kv");
    if !db_dir.exists() {
        fs::create_dir(db_dir).expect("Failed to create db.kv directory");
    }
}

// A utility function to start the kvapp server.
// Returns a Child process handle, which can be used to kill the server later.
fn start_kvapp_server() -> Child {
    // Specify the binary name using "--bin kvapp" parameter to `cargo run`.
    let child = Command::new("cargo")
        .args(["run", "--bin", "kvapp"])
        .spawn()
        .expect("Failed to start kvapp server");

    // Give the server some time to start up.
    thread::sleep(Duration::from_secs(DEF_START_WAIT));

    child
}

// A utility function to stop the kvapp server.
fn stop_kvapp_server(mut child: Child) {
    child.kill().expect("Failed to kill kvapp server");
}

// Example of an integration test that starts the server, makes a request, and stops the server.
#[tokio::test]
async fn test_kvapp_integration() {
    // Prepare server environment
    prepare_environment();

    // Start the server in the background.
    let server_process = start_kvapp_server();

    // Create HTTP client
    let client = reqwest::Client::new();

    // ----------------------------------------------------------------
    // Test: index
    let res = client
        .get("http://localhost:8080/")
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success(), "Request did not succeed");

    // Deserialize the response body to a JSON Value and assert "healthy" is true.
    let json: Value = res.json().await.expect("Failed to deserialize JSON");
    assert_eq!(json["name"], "kvapp");

    // ----------------------------------------------------------------
    // Test: health check
    let res = client
        .get("http://localhost:8080/health")
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success(), "Request did not succeed");

    // Deserialize the response body to a JSON Value and assert "healthy" is true.
    let json: Value = res.json().await.expect("Failed to deserialize JSON");
    assert_eq!(json["healthy"], true, "Server is not healthy");

    // ----------------------------------------------------------------
    // Test: Get non-existent object returns not-found
    let url = "http://localhost:8080/api/1";
    let res = client
        .get(url)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);

    // ----------------------------------------------------------------
    // Test: Delete non-existent object returns not-found
    let res = client
        .delete(url)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);

    // ----------------------------------------------------------------
    // Test: Put a new object
    let res = client
        .put(url)
        .body(T_VALUE)
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success(), "Request did not succeed");

    // ----------------------------------------------------------------
    // Test: Get just-stored object, validate contents match
    let res = client
        .get(url)
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success(), "Request did not succeed");

    // Check body text match
    let body_text = res.text().await.expect("Failed to receive text");
    assert_eq!(body_text, T_VALUE);

    // ----------------------------------------------------------------
    // Test: Delete just-stored object
    let res = client
        .delete(url)
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success(), "Request did not succeed");

    // ----------------------------------------------------------------
    // Test (again): Get non-existent object returns not-found
    let res = client
        .get(url)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);

    // ----------------------------------------------------------------
    // Test (again): Delete non-existent object returns not-found
    let res = client
        .delete(url)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);

    // Stop the server.
    stop_kvapp_server(server_process);
}

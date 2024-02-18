use std::env;
use std::fs;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

// A utility function to prepare the environment before starting the server.
fn prepare_environment() {
    // Use CARGO_MANIFEST_DIR to get the path to the source directory
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Construct the source path for the configuration file
    let config_src_path = Path::new(&cargo_dir).join("example-cfg-kvapp.json");
    // Define the destination path for the configuration file
    let config_dest_path = Path::new("cfg-kvapp.json");

    // Copy the configuration file to the current directory.
    fs::copy(config_src_path, config_dest_path)
        .expect("Failed to copy configuration file");

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
    thread::sleep(Duration::from_secs(5));

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

    // Perform your test logic here.
    // Example: Issue an HTTP GET request to the server.
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:8080/health")
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success(), "Request did not succeed");

    // Additional assertions can be made here based on the response.

    // Stop the server.
    stop_kvapp_server(server_process);
}

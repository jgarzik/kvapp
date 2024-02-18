/*
 * tester: Integration tester for kvapp
 *
 * To be run separately from kvapp, assuming a clean and empty db:
 * $ cargo run --bin kvapp
 * $ cargo run --bin tester
 */

extern crate reqwest;

const T_ENDPOINT: &'static str = "http://127.0.0.1:8080";
const T_BASEURI: &'static str = "/api/db/";
const T_VALUE: &'static str = "helloworld";

use reqwest::{Client, StatusCode};

async fn post_get_put_get() -> Result<(), reqwest::Error> {
    let basepath = format!("{}{}", T_ENDPOINT, T_BASEURI);

    let client = Client::new();

    // Check that a record with key 1 doesn't exist.
    let url = format!("{}1", basepath);
    let resp_res = client.get(&url).send().await;
    match resp_res {
        Ok(resp) => assert_eq!(resp.status(), StatusCode::NOT_FOUND),
        Err(_e) => assert!(false),
    }

    // verify DELETE(non exist) returns not-found
    let resp_res = client.delete(&url).send().await;
    match resp_res {
        Ok(resp) => assert_eq!(resp.status(), StatusCode::NOT_FOUND),
        Err(_e) => assert!(false),
    }

    // PUT a new record
    let resp_res = client.put(&url).body(T_VALUE).send().await;
    match resp_res {
        Ok(resp) => assert_eq!(resp.status(), StatusCode::OK),
        Err(_e) => assert!(false),
    }

    // Check that the record exists with the correct contents.
    let resp_res = client.get(&url).send().await;
    match resp_res {
        Ok(resp) => {
            assert_eq!(resp.status(), StatusCode::OK);

            match resp.text().await {
                Ok(body) => assert_eq!(body, T_VALUE),
                Err(_e) => assert!(false),
            }
        }
        Err(_e) => assert!(false),
    }

    // DELETE record
    let resp_res = client.delete(&url).send().await;
    match resp_res {
        Ok(resp) => assert_eq!(resp.status(), StatusCode::OK),
        Err(_e) => assert!(false),
    }

    // Check (again) that a record with key 1 doesn't exist.
    let resp_res = client.get(&url).send().await;
    match resp_res {
        Ok(resp) => assert_eq!(resp.status(), StatusCode::NOT_FOUND),
        Err(_e) => assert!(false),
    }

    // verify (again) DELETE(non exist) returns not-found
    let resp_res = client.delete(&url).send().await;
    match resp_res {
        Ok(resp) => assert_eq!(resp.status(), StatusCode::NOT_FOUND),
        Err(_e) => assert!(false),
    }

    // test health check endpoint
    let health_url = format!("{}/health", T_ENDPOINT);
    let resp_res = client.get(&health_url).send().await;
    match resp_res {
        Ok(resp) => assert_eq!(resp.status(), StatusCode::OK),
        Err(_e) => assert!(false),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let f_res = post_get_put_get().await;
    match f_res {
        Ok(_f) => {
            println!("Integration testing successful.");
            Ok(())
        }
        Err(e) => {
            println!("Integration testing FAILED.");
            Err(e)
        }
    }
}

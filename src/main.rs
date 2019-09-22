
#[macro_use] extern crate actix_web;
extern crate clap;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DEF_DB_DIR: &'static str = "db.kv";
const DEF_BIND_ADDR: &'static str = "127.0.0.1";
const DEF_BIND_PORT: &'static str = "8080";

use std::{env, io, fmt};

use actix_web::http::{StatusCode};
use actix_web::{
    guard, middleware, web, App, HttpRequest, HttpResponse, HttpServer,
    Result,
};
use serde_json::json;
use sled::{Db,ConfigBuilder};

struct ServerState {
    db: Db
}

// helper function, 404 not found
fn err_not_found() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("application/json")
        .body(json!({
          "error": {
             "code" : -404,
              "message": "not found"}}).to_string()))
}

// helper function, server error
fn err_500() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("application/json")
        .body(json!({
          "error": {
             "code" : -500,
              "message": "internal server error"}}).to_string()))
}

// helper function, success + json response
fn ok_json(jval: serde_json::Value) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json")
        .body(jval.to_string()))
}

/// simple root index handler, describes our service
#[get("/")]
fn index(state: web::Data<ServerState>, req: HttpRequest) -> Result<HttpResponse> {
    println!("{:?}", req);

    ok_json(json!({
            "name": "kvapp",
            "version": VERSION}))
}

/// GET data item.  key in URI path.  returned value as json response
fn get(state: web::Data<ServerState>, req: HttpRequest, path: web::Path<(String,)>) -> Result<HttpResponse> {
    println!("{:?}", req);

    match state.db.get(path.0.clone()) {
        Ok(optval) => match optval {
            Some(val) => ok_json(json!({"result": String::from_utf8(val.to_vec()).unwrap()})),
            None => err_not_found()     // db: value not found
        },
        Err(_e) => err_500()            // db: error
    }
}

/// PUT data item.  key and value both in URI path.
fn put(state: web::Data<ServerState>, req: HttpRequest, path: web::Path<(String,String)>) -> Result<HttpResponse> {
    println!("{:?}", req);

    match state.db.insert(path.0.as_str(), path.1.as_str()) {
        Ok(_optval) => ok_json(json!({"result": true})),
        Err(_e) => err_500()            // db: error
    }
}

/// 404 handler
fn p404() -> Result<HttpResponse> {
    err_not_found()
}

fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // parse command line
    let cli_matches = clap::App::new("kvapp")
                      .version(VERSION)
                      .author("Jeff Garzik <jgarzik@pobox.com>")
                      .about("Database server for key/value db")
                      .arg(clap::Arg::with_name("db")
                           .long("db")
                           .value_name("DIR")
                           .help("Sets a custom database directory (default: db.kv)")  // best way to include DEF_DB_DIR in help string?
                           .takes_value(true))
                      .arg(clap::Arg::with_name("bind-addr")
                           .long("bind-addr")
                           .value_name("IP-ADDRESS")
                           .help("Custom server socket bind address (default: 127.0.0.1)")  // best way to include DEF_DB_DIR in help string?
                           .takes_value(true))
                      .arg(clap::Arg::with_name("bind-port")
                           .long("bind-port")
                           .value_name("PORT")
                           .help("Custom server socket bind port (default: 8080)")  // best way to include DEF_DB_DIR in help string?
                           .takes_value(true))
                      .get_matches();

    // configure based on CLI options
    let db_dir = cli_matches.value_of("config").unwrap_or(DEF_DB_DIR);
    let bind_addr = cli_matches.value_of("bind-addr").unwrap_or(DEF_BIND_ADDR);
    let bind_port = cli_matches.value_of("bind-port").unwrap_or(DEF_BIND_PORT);
    let mut bind_pair = String::new();
    fmt::write(&mut bind_pair, format_args!("{}:{}", bind_addr, bind_port))
        .expect("[Insert wasteful error message for never-occur warning]");

    // configure & open db
    let db_config = ConfigBuilder::default()
        .path(db_dir)
        .use_compression(false);
    let db = Db::start(db_config.build()).unwrap();

    // configure web server
    let sys = actix_rt::System::new("kvapp");

    HttpServer::new(move || {
        App::new()
            // pass application state to each handler
            .data(ServerState { db: db.clone() })

            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())

            // register our routes
            .service(index)
            .service(
                web::resource("/1/db/{dbkey}")
                    .route(web::get().to(get))
            )
            .service(
                web::resource("/1/db/{dbkey}/{dbval}")
                    .route(web::put().to(put))
            )

            // default
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET` -- redundant?
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind(bind_pair.to_string())?
    .start();

    println!("Starting http server: {}", bind_pair);
    sys.run()
}

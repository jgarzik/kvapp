#[macro_use]
extern crate actix_web;
extern crate clap;

const APPNAME: &'static str = "kvapp";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DEF_CFG_FN: &'static str = "cfg-kvapp.json";
const DEF_DB_NAME: &'static str = "db";
const DEF_DB_DIR: &'static str = "db.kv";
const DEF_BIND_ADDR: &'static str = "127.0.0.1";
const DEF_BIND_PORT: &'static str = "8080";

use std::sync::{Arc, Mutex};
use std::{env, fs};

use actix_web::http::StatusCode;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use sled::Db;

#[derive(Serialize, Deserialize)]
struct DbConfig {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct ServerConfig {
    databases: Vec<DbConfig>,
}

struct ServerState {
    name: String, // db nickname
    db: Db,       // open db handle
}

// helper function, 404 not found
fn err_not_found() -> HttpResponse {
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("application/json")
        .body(
            json!({
          "error": {
             "code" : -404,
              "message": "not found"}})
            .to_string(),
        )
}

// helper function, server error
fn err_500() -> HttpResponse {
    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("application/json")
        .body(
            json!({
          "error": {
             "code" : -500,
              "message": "internal server error"}})
            .to_string(),
        )
}

// helper function, success + binary response
fn ok_binary(val: Vec<u8>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(val)
}

// helper function, success + json response
fn ok_json(jval: serde_json::Value) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(jval.to_string())
}

/// simple root index handler, describes our service
#[get("/")]
async fn req_index(m_state: web::Data<Arc<Mutex<ServerState>>>) -> HttpResponse {
    let state = m_state.lock().unwrap();

    ok_json(json!({
        "name": APPNAME,
        "version": VERSION,
        "databases": [
            { "name": state.name }
        ]
    }))
}

/// DELETE data item.  key in URI path.  returned ok as json response
async fn req_delete(
    m_state: web::Data<Arc<Mutex<ServerState>>>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let state = m_state.lock().unwrap();

    // we only support 1 db, for now...  user must specify db name
    if state.name != path.0 {
        return err_not_found();
    }

    match state.db.remove(path.1.clone()) {
        Ok(optval) => match optval {
            Some(_val) => ok_json(json!({"result": true})),
            None => err_not_found(), // db: value not found
        },
        Err(_e) => err_500(), // db: error
    }
}

/// GET data item.  key in URI path.  returned value as json response
async fn req_get(
    m_state: web::Data<Arc<Mutex<ServerState>>>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let state = m_state.lock().unwrap();

    // we only support 1 db, for now...  user must specify db name
    if state.name != path.0 {
        return err_not_found();
    }

    match state.db.get(path.1.clone()) {
        Ok(optval) => match optval {
            Some(val) => ok_binary(val.to_vec()),
            None => err_not_found(), // db: value not found
        },
        Err(_e) => err_500(), // db: error
    }
}

/// PUT data item.  key and value both in URI path.
async fn req_put(
    m_state: web::Data<Arc<Mutex<ServerState>>>,
    (path, body): (web::Path<(String, String)>, web::Bytes),
) -> HttpResponse {
    let state = m_state.lock().unwrap();

    // we only support 1 db, for now...  user must specify db name
    if state.name != path.0 {
        return err_not_found();
    }

    match state.db.insert(path.1.as_str(), body.to_vec()) {
        Ok(_optval) => ok_json(json!({"result": true})),
        Err(_e) => err_500(), // db: error
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // parse command line
    let cli_matches = clap::Command::new(APPNAME)
        .about("Database server for key/value db")
        .version(VERSION)
        .author("Jeff Garzik <jgarzik@pobox.com>")
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("Sets a custom configuration file")
                .default_value(DEF_CFG_FN)
                .value_name("JSON-FILE")
                .takes_value(true),
        )
        .arg(
            clap::Arg::new("bind-addr")
                .long("bind-addr")
                .help("Custom server socket bind address")
                .default_value(DEF_BIND_ADDR)
                .value_name("IP-ADDRESS")
                .takes_value(true),
        )
        .arg(
            clap::Arg::new("bind-port")
                .long("bind-port")
                .help("Custom server socket bind port")
                .default_value(DEF_BIND_PORT)
                .value_name("PORT")
                .takes_value(true),
        )
        .get_matches();

    // configure based on CLI options
    let bind_addr = cli_matches.value_of("bind-addr").unwrap_or(DEF_BIND_ADDR);
    let bind_port = cli_matches.value_of("bind-port").unwrap_or(DEF_BIND_PORT);
    let bind_pair = format!("{}:{}", bind_addr, bind_port);
    let server_hdr = format!("{}/{}", APPNAME, VERSION);

    // read JSON configuration file
    let cfg_fn = cli_matches.value_of("config").unwrap_or(DEF_CFG_FN);
    let cfg_text = fs::read_to_string(cfg_fn)?;
    let server_cfg: ServerConfig = serde_json::from_str(&cfg_text)?;

    // special case, until we have multiple dbs: find first db config, use it
    let db_name;
    let db_path;
    if server_cfg.databases.len() == 0 {
        db_name = String::from(DEF_DB_NAME);
        db_path = String::from(DEF_DB_DIR);
    } else {
        db_name = server_cfg.databases[0].name.clone();
        db_path = server_cfg.databases[0].path.clone();
    }

    // configure & open db
    let db_config = sled::Config::default().path(db_path).use_compression(false);
    let db = db_config.open().unwrap();

    let srv_state = Arc::new(Mutex::new(ServerState {
        name: db_name.clone(),
        db: db.clone(),
    }));

    // configure web server
    println!("Starting http server: {}", bind_pair);
    HttpServer::new(move || {
        App::new()
            // pass application state to each handler
            .app_data(web::Data::new(Arc::clone(&srv_state)))
            // apply default headers
            .wrap(middleware::DefaultHeaders::new().add(("Server", server_hdr.to_string())))
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register our routes
            .service(req_index)
            .service(
                web::resource("/api/{db}/{key}")
                    .route(web::get().to(req_get))
                    .route(web::put().to(req_put))
                    .route(web::delete().to(req_delete)),
            )
    })
    .bind(bind_pair.to_string())?
    .run()
    .await
}

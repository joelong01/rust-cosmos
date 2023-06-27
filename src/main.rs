/**
 *  main entry point for the application.  The goal here is to set up the Web Server.
 */
//
//  rust wants modules in the same directory declared.
mod cosmosdb;
mod models;
mod users;
mod utility;

// dependencies...
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use cosmosdb::get_cosmos_secrets;
use log::{error, trace};
use once_cell::sync::OnceCell;
use opentelemetry::{
    global, runtime::TokioCurrentThread, sdk::propagation::TraceContextPropagator,
};
use std::env;
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/**
 *  Code to pick a port in a threadsafe way -- either specified in an environment variable named COSMOS_RUST_SAMPLE_PORT
 *  or 8080
 */
static PORT: OnceCell<String> = OnceCell::new();

#[allow(unused_macros)]
#[macro_export]
macro_rules! safe_set_port {
    () => {{
        let port: String;
        match PORT.get() {
            Some(val) => {
                port = val.to_string();
            }
            None => {
                match env::var("COSMOS_RUST_SAMPLE_PORT") {
                    Ok(val) => port = val.to_string(),
                    Err(_e) => port = "8080".to_string(),
                }
                println!("setting port to: {}", port);
                match PORT.set(port.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error setting port: {:?}", e.to_string());
                    }
                }
            }
        };
        port
    }};
}
/**
 *  main:  entry point that sets up the web service
 */
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //
    //   turn on max logging by setting web, server, and rust to trace level logging

    env::set_var("RUST_LOG", "actix_web=trace,actix_server=trace,rust=trace");
    init_telemetry();

    let port: String = safe_set_port!();
    // this looks up env variables and puts them into a rust structt - if they aren't set, we error out
    let secrets = get_cosmos_secrets();
    match secrets {
        Ok(secrets) => trace!("Secrets found.  Account: {:?}", secrets.account),
        Err(error) => error!("Failed to get secrets: {}", error),
    }

    // normally you would set the RUST_LOG environment variable in the process that this app is running in, and then
    // you'd have this code that checks for it and errors out if it doesn't exist.  this value is set above, so it will
    // always be set, but this is here for completeness
    let rust_log = env::var("RUST_LOG");
    match rust_log {
        Ok(value) => trace!("RUST_LOG: {}", value),
        Err(_) => trace!("RUST_LOG is not set"),
    }

    //
    // set up the HttpServer

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api").service(
                    web::scope("/v1")
                        .route("/users", web::get().to(users::list_users))
                        .route("/users", web::post().to(users::create))
                        .route("/users/{id}", web::delete().to(users::delete))
                        .route("/users/{id}", web::get().to(users::find_user_by_id))
                        .route("/setup", web::post().to(users::setup)),
                ),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}

//
fn init_telemetry() {
    let app_name = "cosmos-rust";

    // Start a new Jaeger trace pipeline. Spans are exported in batches.
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(app_name)
        .install_batch(TokioCurrentThread)
        .expect("Failed to install OpenTelemetry tracer.");

    // Tunable via `RUST_LOG` env variable
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let formatting_layer = BunyanFormattingLayer::new(app_name.into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.")
}

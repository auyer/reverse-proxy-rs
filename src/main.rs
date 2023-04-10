#![feature(ip)]
#![feature(is_some_and)]

use axum::{middleware, routing::any, Router};
use hyper::Client;
use hyper_tls::HttpsConnector;
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod configuration;
mod filter;
mod handlers;
mod shutdown;
mod templates;

use configuration::read_configuration;

#[tokio::main]
async fn main() {
    let config = read_configuration();

    let mut level = Level::DEBUG;
    if config.debug {
        // level = Level::DEBUG;
        level = Level::TRACE;
    }
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tokio::join!(server(config.port));
}

async fn server(port: u16) {
    let https = HttpsConnector::new();
    let https_client = Client::builder().build::<_, hyper::Body>(https);

    let app = Router::new()
        .route("/", any(handlers::home_page))
        .route("/*key", any(handlers::proxy_handler))
        .with_state(https_client)
        .layer(middleware::from_fn(filter::filter_internal_ips))
        .layer(TimeoutLayer::new(Duration::from_secs(1000)));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!("reverse proxy listening on{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await
        .expect("server failed");
}

async fn https_server(port: u16) {
    let https = HttpsConnector::new();
    let https_client = Client::builder().build::<_, hyper::Body>(https);

    let app = Router::new()
        .route("/", any(handlers::home_page))
        .route("/*key", any(handlers::proxy_handler))
        .with_state(https_client)
        .layer(middleware::from_fn(filter::filter_internal_ips))
        .layer(TimeoutLayer::new(Duration::from_nanos(10)));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    // println!("reverse proxy listening on {}", addr);
    tracing::info!("reverse proxy listening on{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await
        .expect("server failed");
}

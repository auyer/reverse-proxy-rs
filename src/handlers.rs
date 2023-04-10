use super::templates;
use axum::{
    extract::{ConnectInfo, State},
    http::{header::HeaderName, uri::Uri, Request},
    response::{Html, Response},
};
use hyper::{client::HttpConnector, Body};
use hyper_tls::HttpsConnector;
use std::net::SocketAddr;

// type HttpClient = hyper::client::Client<HttpConnector, Body>;
type HttpsClient = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;

pub async fn proxy_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(https_client): State<HttpsClient>,
    req: Request<Body>,
) -> Response<Body> {
    return proxy_handler_retry(addr, https_client, req).await;
}

async fn proxy_handler_retry(
    addr: SocketAddr,
    https_client: HttpsClient,
    mut req: Request<Body>,
) -> Response<Body> {
    let path = req.uri().path();

    let mut path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    path_query = path_query.strip_prefix("/").unwrap();
    // println!("{}", path_query);
    tracing::info!("{}", path_query);

    let uri = format!("https://{}", path_query);

    *req.uri_mut() = Uri::try_from(uri).unwrap();
    let headers = req.headers_mut();

    let XForwardedForHeaderName = HeaderName::from_bytes(b"X-Forwarded-For").unwrap();
    // check if X-Forwarded-For exists
    // set X-Forwarded-For header
    if headers.get(XForwardedForHeaderName.clone()).is_none() {
        headers.insert(
            XForwardedForHeaderName,
            addr.ip().to_string().parse().unwrap(),
        );
    }

    // using Hyper client makes it easy to forward responses back because Axum uses hyper as well.
    // the fact that the client request type is the same as the server, it a delight.
    https_client.request(req).await.unwrap()
}

pub async fn home_page() -> Html<&'static str> {
    Html(templates::HTML_CONTENT)
}

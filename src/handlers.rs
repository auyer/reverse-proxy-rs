use super::templates;
use axum::{
    extract::{ConnectInfo, State},
    http::{header::HeaderName, uri::Uri, Request},
    response::{Html, Response},
};
use hyper::{client::HttpConnector, Body};
use hyper_tls::HttpsConnector;
use std::net::SocketAddr;
use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::follow_redirect::{FollowRedirect, FollowRedirectLayer};

// type HttpClient = hyper::client::Client<HttpConnector, Body>;
type HttpsClient = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;

fn build_follow_redirect_client(client: HttpsClient) -> FollowRedirect<HttpsClient> {
    // ) -> FollowRedirect<tower::util::MapErr<HttpsClient, ProxyError>> {
    // let policy = policy::Limited::new(10) // Set the maximum number of redirections to 10.
    //     // Return an error when the limit was reached.
    //     .or::<_, (), _>(policy::redirect_fn(|_| Err(ProxyError::TooManyRedirects)))
    //     // Do not follow cross-origin redirections, and return the redirection responses as-is.
    //     .and::<_, (), _>(policy::SameOrigin::new());

    let client = ServiceBuilder::new()
        .layer(FollowRedirectLayer::new())
        // .layer(FollowRedirectLayer::with_policy(policy))
        // .map_err(ProxyError::Hyper)
        .service(client);
    return client;
}

pub async fn proxy_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(https_client): State<HttpsClient>,
    mut req: Request<Body>,
) -> Response<Body> {
    // REWRITE URI
    let path = req.uri().path();

    let path = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path)
        .strip_prefix("/")
        .unwrap();

    let uri = format!("https://{}", path);
    tracing::info!("Going to request {}", uri);

    *req.uri_mut() = Uri::try_from(uri).unwrap();
    let headers = req.headers_mut();

    let xforwarded_for_header_name = HeaderName::from_bytes(b"X-Forwarded-For").unwrap();
    // check if X-Forwarded-For exists
    // set X-Forwarded-For header
    if headers.get(xforwarded_for_header_name.clone()).is_none() {
        headers.insert(
            xforwarded_for_header_name,
            addr.ip().to_string().parse().unwrap(),
        );
    }
    let mut client = build_follow_redirect_client(https_client);

    // using Hyper client makes it easy to forward responses back because Axum uses hyper as well.
    // the fact that the client request type is the same as the server, it a delight.
    return client.ready().await.unwrap().call(req).await.unwrap();
}

pub async fn home_page() -> Html<&'static str> {
    Html(templates::HTML_CONTENT)
}

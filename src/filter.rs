use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::net::IpAddr;
use std::str::FromStr;

pub async fn filter_internal_ips<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    // this part will get the uri and re-parse as a full URL.
    let mut uri = req.uri().to_string();

    let uri_stripped = uri.strip_prefix("/");

    if uri_stripped.is_some_and(|u| u.len() > 0) {
        uri = uri_stripped.unwrap().to_string();
    }

    let uri_parts: Vec<&str> = uri.split("/").collect();

    if uri_parts.len() > 0 && uri_parts[0].contains(":") {
        let uri_parts: Vec<&str> = uri_parts[0].split(":").collect();
        if uri_parts.len() > 0 {
            uri = uri_parts[0].to_string();
        }
    } else if uri_parts.len() > 0 {
        uri = uri_parts[0].to_string();
    }

    // try to read host ad IP. Is successfull, will exclude local IPs
    let addr = IpAddr::from_str(&uri);
    if addr.is_ok() {
        let addr = addr.ok().unwrap();
        if !addr.is_global() || addr.is_loopback() {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    // also removes localhost.
    if uri.starts_with("localhost") {
        return Err(StatusCode::FORBIDDEN);
    }

    return Ok(next.run(req).await);
}

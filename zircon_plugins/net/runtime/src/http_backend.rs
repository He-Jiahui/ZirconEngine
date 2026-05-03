use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use zircon_runtime::core::framework::net::{
    NetError, NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetHttpRouteDescriptor, NetRequestId, NetRouteId,
};

#[derive(Debug)]
pub(crate) struct ManagedHttpListener {
    pub local_endpoint: zircon_runtime::core::framework::net::NetEndpoint,
}

#[derive(Debug)]
pub(crate) struct ManagedHttpRoute {
    pub route: NetHttpRouteDescriptor,
    pub response: NetHttpResponseDescriptor,
}

pub(crate) fn path_from_http_url(url: &str) -> String {
    if let Some((_, rest)) = url.split_once("://") {
        match rest.find('/') {
            Some(index) => rest[index..].to_string(),
            None => "/".to_string(),
        }
    } else if url.starts_with('/') {
        url.to_string()
    } else {
        format!("/{url}")
    }
}

pub(crate) fn url_has_explicit_port(url: &str) -> bool {
    let Some((_, rest)) = url.split_once("://") else {
        return false;
    };
    let authority = rest.split('/').next().unwrap_or_default();
    authority.rsplit_once(':').is_some_and(|(_, port)| {
        !port.is_empty() && port.chars().all(|character| character.is_ascii_digit())
    })
}

pub(crate) async fn send_http_request(
    request: NetHttpRequestDescriptor,
) -> Result<NetHttpResponseDescriptor, NetError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(request.timeout_ms))
        .use_rustls_tls()
        .build()
        .map_err(|error| NetError::Io(error.to_string()))?;
    let mut builder = client.request(method_to_reqwest(request.method), &request.url);
    for (name, value) in &request.headers {
        builder = builder.header(name, value);
    }
    if !request.body.is_empty() {
        builder = builder.body(request.body.clone());
    }
    let response = builder
        .send()
        .await
        .map_err(|error| NetError::Io(error.to_string()))?;
    let status_code = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect::<Vec<_>>();
    let body = response
        .bytes()
        .await
        .map_err(|error| NetError::Io(error.to_string()))?
        .to_vec();
    let mut response = NetHttpResponseDescriptor::new(request.request, status_code, body);
    response.headers = headers;
    Ok(response)
}

pub(crate) async fn serve_http_listener(
    listener: TcpListener,
    routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
) {
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(_) => return,
        };
        let routes = routes.clone();
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |request| handle_route_request(request, routes.clone()));
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await;
        });
    }
}

async fn handle_route_request(
    request: Request<Incoming>,
    routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let method = http_method_from_hyper(request.method());
    let path = request.uri().path().to_string();
    let matched = method.and_then(|method| {
        routes
            .lock()
            .expect("net HTTP routes mutex poisoned")
            .values()
            .find(|entry| entry.route.path == path && entry.route.methods.contains(&method))
            .map(|entry| entry.response.clone().for_request(NetRequestId::new(0)))
    });

    let response = match matched {
        Some(response) => {
            let mut builder = Response::builder().status(response.status_code);
            for (name, value) in response.headers {
                builder = builder.header(name, value);
            }
            builder
                .body(Full::new(Bytes::from(response.body)))
                .unwrap_or_else(|_| internal_server_error())
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from_static(b"route not found")))
            .unwrap_or_else(|_| internal_server_error()),
    };

    Ok(response)
}

fn internal_server_error() -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::from_static(b"internal server error")))
        .expect("static HTTP response should build")
}

pub(crate) fn method_to_reqwest(method: NetHttpMethod) -> reqwest::Method {
    match method {
        NetHttpMethod::Get => reqwest::Method::GET,
        NetHttpMethod::Post => reqwest::Method::POST,
        NetHttpMethod::Put => reqwest::Method::PUT,
        NetHttpMethod::Patch => reqwest::Method::PATCH,
        NetHttpMethod::Delete => reqwest::Method::DELETE,
    }
}

fn http_method_from_hyper(method: &hyper::Method) -> Option<NetHttpMethod> {
    if method == hyper::Method::GET {
        Some(NetHttpMethod::Get)
    } else if method == hyper::Method::POST {
        Some(NetHttpMethod::Post)
    } else if method == hyper::Method::PUT {
        Some(NetHttpMethod::Put)
    } else if method == hyper::Method::PATCH {
        Some(NetHttpMethod::Patch)
    } else if method == hyper::Method::DELETE {
        Some(NetHttpMethod::Delete)
    } else {
        None
    }
}

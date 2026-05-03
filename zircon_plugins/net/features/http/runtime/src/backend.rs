use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use zircon_plugin_net_runtime::{HttpRuntimeBackend, ManagedHttpListener, ManagedHttpRoute};
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetRequestId, NetRouteId,
};

#[derive(Clone, Debug, Default)]
pub struct HyperReqwestHttpBackend;

pub fn http_runtime_backend() -> Arc<dyn HttpRuntimeBackend> {
    Arc::new(HyperReqwestHttpBackend)
}

impl HttpRuntimeBackend for HyperReqwestHttpBackend {
    fn listen_http(
        &self,
        runtime: &tokio::runtime::Runtime,
        bind: SocketAddr,
        routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
    ) -> Result<ManagedHttpListener, NetError> {
        let listener = runtime
            .block_on(TcpListener::bind(bind))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_endpoint = listener
            .local_addr()
            .map(NetEndpoint::from)
            .map_err(|error| NetError::Io(error.to_string()))?;
        let abort_handle = runtime
            .spawn(serve_http_listener(listener, routes))
            .abort_handle();
        Ok(ManagedHttpListener {
            local_endpoint,
            abort_handle: Some(abort_handle),
        })
    }

    fn send_http_request(
        &self,
        runtime: &tokio::runtime::Runtime,
        request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError> {
        runtime.block_on(send_http_request(request))
    }
}

async fn send_http_request(
    request: NetHttpRequestDescriptor,
) -> Result<NetHttpResponseDescriptor, NetError> {
    validate_http_security_policy(&request)?;
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

fn validate_http_security_policy(request: &NetHttpRequestDescriptor) -> Result<(), NetError> {
    if request.security.certificate_pinning {
        return Err(NetError::SecurityPolicyViolation {
            reason: "HTTP certificate pinning is not configured".to_string(),
        });
    }

    if request.security.tls_required
        && !request.url.starts_with("https://")
        && !(request.security.allow_insecure_loopback && http_url_is_loopback(&request.url))
    {
        return Err(NetError::SecurityPolicyViolation {
            reason: "HTTP request requires HTTPS by security policy".to_string(),
        });
    }

    Ok(())
}

fn http_url_is_loopback(url: &str) -> bool {
    let Some(authority) = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .map(|rest| rest.split('/').next().unwrap_or_default())
    else {
        return false;
    };
    let host = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority)
        .split(':')
        .next()
        .unwrap_or_default();
    matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]")
}

async fn serve_http_listener(
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
            .map(|entry| {
                let request =
                    NetHttpRequestDescriptor::new(NetRequestId::new(0), method, path.clone());
                entry
                    .handler
                    .as_ref()
                    .map(|handler| handler(request.clone()))
                    .unwrap_or_else(|| entry.response.clone().for_request(request.request))
            })
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

fn method_to_reqwest(method: NetHttpMethod) -> reqwest::Method {
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

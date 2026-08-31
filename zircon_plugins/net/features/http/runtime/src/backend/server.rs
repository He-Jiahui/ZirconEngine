use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use http_body_util::{BodyExt, Full, LengthLimitError, Limited};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use zircon_plugin_net_runtime::{ManagedHttpListener, ManagedHttpRoute};
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetRequestId,
    NetRouteId,
};

use super::method::http_method_from_hyper;
use super::HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES;

pub(super) fn listen_http(
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
            .map(|entry| (method, entry.response.clone(), entry.handler.clone()))
    });
    let Some((method, route_response, route_handler)) = matched else {
        // Keep the HTTP/1 request lifecycle valid without buffering an unmatched payload.
        if discard_route_request_body(request.into_body())
            .await
            .is_err()
        {
            return Ok(internal_server_error());
        }
        return Ok(route_not_found());
    };

    let headers = request
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect::<Vec<_>>();
    let body = match collect_route_request_body(request.into_body()).await {
        Ok(body) => body,
        Err(RouteBodyError::TooLarge) => return Ok(payload_too_large()),
        Err(RouteBodyError::ReadFailed) => return Ok(internal_server_error()),
    };
    let mut request = NetHttpRequestDescriptor::new(NetRequestId::new(0), method, path);
    request.headers = headers;
    request.body = body;
    let request_id = request.request;
    let response = match route_handler {
        Some(handler) => handler(request),
        None => route_response.for_request(request_id),
    };

    build_route_response(response)
}

fn build_route_response(
    response: NetHttpResponseDescriptor,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let mut builder = Response::builder().status(response.status_code);
    for (name, value) in response.headers {
        builder = builder.header(name, value);
    }
    let response = builder
        .body(Full::new(Bytes::from(response.body)))
        .unwrap_or_else(|_| internal_server_error());

    Ok(response)
}

enum RouteBodyError {
    TooLarge,
    ReadFailed,
}

async fn collect_route_request_body(body: Incoming) -> Result<Vec<u8>, RouteBodyError> {
    Limited::new(body, HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES)
        .collect()
        .await
        .map(|collected| collected.to_bytes().to_vec())
        .map_err(|error| {
            if error.downcast_ref::<LengthLimitError>().is_some() {
                RouteBodyError::TooLarge
            } else {
                RouteBodyError::ReadFailed
            }
        })
}

async fn discard_route_request_body(mut body: Incoming) -> Result<(), RouteBodyError> {
    while let Some(frame) = body.frame().await {
        frame.map_err(|_| RouteBodyError::ReadFailed)?;
    }
    Ok(())
}

fn route_not_found() -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from_static(b"route not found")))
        .expect("static HTTP response should build")
}

fn payload_too_large() -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::PAYLOAD_TOO_LARGE)
        .body(Full::new(Bytes::from_static(b"request body too large")))
        .expect("static HTTP response should build")
}

fn internal_server_error() -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::from_static(b"internal server error")))
        .expect("static HTTP response should build")
}

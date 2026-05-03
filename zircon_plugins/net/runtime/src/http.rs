use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;
use zircon_runtime::core::framework::net::{
    NetError, NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetHttpRouteDescriptor,
    NetRouteId,
};

#[derive(Clone, Debug)]
pub struct ManagedHttpListener {
    pub local_endpoint: zircon_runtime::core::framework::net::NetEndpoint,
    pub abort_handle: Option<tokio::task::AbortHandle>,
}

pub struct ManagedHttpRoute {
    pub route: NetHttpRouteDescriptor,
    pub response: NetHttpResponseDescriptor,
    pub handler: Option<HttpRouteHandler>,
}

pub type HttpRouteHandler =
    Arc<dyn Fn(NetHttpRequestDescriptor) -> NetHttpResponseDescriptor + Send + Sync>;

impl fmt::Debug for ManagedHttpRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManagedHttpRoute")
            .field("route", &self.route)
            .field("response", &self.response)
            .field("has_handler", &self.handler.is_some())
            .finish()
    }
}

pub trait HttpRuntimeBackend: Send + Sync + std::fmt::Debug {
    fn listen_http(
        &self,
        runtime: &Runtime,
        bind: SocketAddr,
        routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
    ) -> Result<ManagedHttpListener, NetError>;

    fn send_http_request(
        &self,
        runtime: &Runtime,
        request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError>;
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

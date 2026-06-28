use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use zircon_plugin_net_runtime::{HttpRuntimeBackend, ManagedHttpListener, ManagedHttpRoute};
use zircon_runtime::core::framework::net::{
    NetError, NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetRouteId,
};

mod client;
mod http1_client_policy;
mod method;
mod security;
mod server;

#[derive(Clone, Debug, Default)]
pub struct HyperReqwestHttpBackend;

pub(crate) const HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES: usize = 1024 * 1024;

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
        server::listen_http(runtime, bind, routes)
    }

    fn send_http_request(
        &self,
        runtime: &tokio::runtime::Runtime,
        request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError> {
        runtime.block_on(client::send_http_request(request))
    }
}

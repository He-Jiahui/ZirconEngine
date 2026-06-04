use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetHttpRouteDescriptor, NetListenerId, NetRequestId, NetRouteId, NetTransportKind,
};

use crate::http::{HttpRouteHandler, ManagedHttpRoute};
use crate::HttpRuntimeBackend;

use super::DefaultNetManager;

impl DefaultNetManager {
    pub fn register_http_route_handler(
        &self,
        route: NetHttpRouteDescriptor,
        handler: impl Fn(NetHttpRequestDescriptor) -> NetHttpResponseDescriptor + Send + Sync + 'static,
    ) -> Result<NetRouteId, NetError> {
        let route_id = self.next_route_id();
        self.state
            .http_routes
            .lock()
            .expect("net HTTP routes mutex poisoned")
            .insert(
                route_id,
                ManagedHttpRoute {
                    route: route.clone(),
                    response: NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, Vec::new()),
                    handler: Some(Arc::new(handler) as HttpRouteHandler),
                },
            );
        self.state.push_event(NetEvent::HttpRouteRegistered {
            route: route_id,
            path: route.path,
            methods: route.methods,
        });
        Ok(route_id)
    }

    pub(in crate::service_types) fn http_backend(
        &self,
    ) -> Result<Arc<dyn HttpRuntimeBackend>, NetError> {
        self.state
            .http_backend
            .lock()
            .expect("net HTTP backend mutex poisoned")
            .clone()
            .ok_or_else(|| NetError::ProtocolUnavailable {
                capability: "runtime.feature.net.http".to_string(),
            })
    }

    pub(in crate::service_types) fn register_http_route_impl(
        &self,
        route: NetHttpRouteDescriptor,
        response: NetHttpResponseDescriptor,
    ) -> Result<NetRouteId, NetError> {
        let route_id = self.next_route_id();
        self.state
            .http_routes
            .lock()
            .expect("net HTTP routes mutex poisoned")
            .insert(
                route_id,
                ManagedHttpRoute {
                    response,
                    route: route.clone(),
                    handler: None,
                },
            );
        self.state.push_event(NetEvent::HttpRouteRegistered {
            route: route_id,
            path: route.path,
            methods: route.methods,
        });
        Ok(route_id)
    }

    pub(in crate::service_types) fn unregister_http_route_impl(
        &self,
        route: NetRouteId,
    ) -> Result<(), NetError> {
        self.state
            .http_routes
            .lock()
            .expect("net HTTP routes mutex poisoned")
            .remove(&route)
            .map(|_| ())
            .ok_or(NetError::UnknownRoute { route })
    }

    pub(in crate::service_types) fn listen_http_impl(
        &self,
        bind: &NetEndpoint,
    ) -> Result<NetListenerId, NetError> {
        let bind_addr = bind.to_socket_addr()?;
        let backend = self.http_backend()?;
        let listener = backend.listen_http(
            &self.state.runtime,
            bind_addr,
            self.state.http_routes.clone(),
        )?;
        let local_endpoint = listener.local_endpoint.clone();
        let listener_id = self.next_listener_id();
        self.state
            .http_listeners
            .lock()
            .expect("net HTTP listeners mutex poisoned")
            .insert(listener_id, listener);
        self.state.push_event(NetEvent::ListenerStarted {
            listener: listener_id,
            transport: NetTransportKind::Http,
            endpoint: local_endpoint,
        });
        Ok(listener_id)
    }

    pub(in crate::service_types) fn send_http_request_impl(
        &self,
        request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError> {
        let path = crate::http::path_from_http_url(&request.url);
        let routes = self
            .state
            .http_routes
            .lock()
            .expect("net HTTP routes mutex poisoned");
        if !crate::http::url_has_explicit_port(&request.url) {
            if let Some(response) = routes
                .values()
                .find(|entry| {
                    entry.route.path == path && entry.route.methods.contains(&request.method)
                })
                .map(|entry| {
                    entry
                        .handler
                        .as_ref()
                        .map(|handler| handler(request.clone()))
                        .unwrap_or_else(|| entry.response.clone().for_request(request.request))
                })
            {
                return Ok(response);
            }
        }
        drop(routes);
        self.http_backend()?
            .send_http_request(&self.state.runtime, request)
    }
}

use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetHttpRouteDescriptor, NetListenerId, NetRequestId, NetRouteId, NetTransportKind,
};

use crate::http::{HttpRouteHandler, ManagedHttpRoute};
use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::HttpRuntimeBackend;

use super::DefaultNetManager;

impl DefaultNetManager {
    pub fn register_http_route_handler(
        &self,
        route: NetHttpRouteDescriptor,
        handler: impl Fn(NetHttpRequestDescriptor) -> NetHttpResponseDescriptor + Send + Sync + 'static,
    ) -> Result<NetRouteId, NetError> {
        let route_id = self.next_route_id();
        lock_or_error(&self.state.http_routes, NetSharedState::HttpRoutes)?.insert(
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
        lock_or_error(&self.state.http_backend, NetSharedState::HttpBackend)?
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
        lock_or_error(&self.state.http_routes, NetSharedState::HttpRoutes)?.insert(
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
        let removed = {
            let mut routes = lock_or_error(&self.state.http_routes, NetSharedState::HttpRoutes)?;
            routes.remove(&route)
        }
        .ok_or(NetError::UnknownRoute { route })?;
        drop(removed);

        self.state
            .push_event(NetEvent::HttpRouteUnregistered { route });
        Ok(())
    }

    pub(in crate::service_types) fn listen_http_impl(
        &self,
        bind: &NetEndpoint,
    ) -> Result<NetListenerId, NetError> {
        let bind_addr = bind.to_socket_addr()?;
        let backend = self.http_backend()?;
        drop(lock_or_error(
            &self.state.http_listeners,
            NetSharedState::HttpListeners,
        )?);
        let listener = backend.listen_http(
            &self.state.runtime,
            bind_addr,
            self.state.http_routes.clone(),
        )?;
        let local_endpoint = listener.local_endpoint.clone();
        let listener_id = self.next_listener_id();
        let mut listeners =
            match lock_or_error(&self.state.http_listeners, NetSharedState::HttpListeners) {
                Ok(listeners) => listeners,
                Err(error) => {
                    if let Some(abort_handle) = &listener.abort_handle {
                        abort_handle.abort();
                    }
                    return Err(error);
                }
            };
        listeners.insert(listener_id, listener);
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
        let outbound_bytes = request.body.len();
        let path = crate::http::path_from_http_url(&request.url);
        let local_route = if !crate::http::url_has_explicit_port(&request.url) {
            lock_or_error(&self.state.http_routes, NetSharedState::HttpRoutes)?
                .values()
                .find(|entry| {
                    entry.route.path == path && entry.route.methods.contains(&request.method)
                })
                .map(|entry| (entry.handler.clone(), entry.response.clone()))
        } else {
            None
        };
        if let Some((handler, response)) = local_route {
            let response = handler
                .map(|handler| handler(request.clone()))
                .unwrap_or_else(|| response.for_request(request.request));
            self.state.record_outbound_bytes(outbound_bytes);
            self.state.record_inbound_bytes(response.body_bytes);
            self.state.record_latency_ms(0);
            return Ok(response);
        }
        let started_at = Instant::now();
        let response = self
            .http_backend()?
            .send_http_request(&self.state.runtime, request)?;
        self.state.record_outbound_bytes(outbound_bytes);
        self.state.record_inbound_bytes(response.body_bytes);
        self.state
            .record_latency_ms(started_at.elapsed().as_millis() as u64);
        Ok(response)
    }
}

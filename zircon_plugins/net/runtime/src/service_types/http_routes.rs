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
            let response = dispatch_local_http_route(handler, response, request);
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

fn dispatch_local_http_route(
    handler: Option<HttpRouteHandler>,
    response: NetHttpResponseDescriptor,
    request: NetHttpRequestDescriptor,
) -> NetHttpResponseDescriptor {
    let request_id = request.request;
    match handler {
        Some(handler) => handler(request),
        None => response.for_request(request_id),
    }
}

#[cfg(test)]
mod moved_local_http_request_tests {
    use std::{hint::black_box, sync::Arc, time::Instant};

    use zircon_runtime::core::framework::net::{
        NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetRequestId,
    };

    use crate::http::HttpRouteHandler;

    use super::dispatch_local_http_route;

    const BENCHMARK_BODY_BYTES: usize = 65_536;
    const BENCHMARK_REQUEST_COUNT: usize = 128;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn moved_local_http_dispatch_preserves_handler_and_fallback_results() {
        let handler = Arc::new(|request: NetHttpRequestDescriptor| {
            NetHttpResponseDescriptor::new(request.request, 201, request.body)
        }) as HttpRouteHandler;
        let dynamic = dispatch_local_http_route(
            Some(handler),
            NetHttpResponseDescriptor::new(NetRequestId::new(0), 500, Vec::new()),
            NetHttpRequestDescriptor::new(
                NetRequestId::new(41),
                NetHttpMethod::Post,
                "http://127.0.0.1/echo",
            )
            .with_body(b"payload".to_vec()),
        );
        assert_eq!(dynamic.request, NetRequestId::new(41));
        assert_eq!(dynamic.status_code, 201);
        assert_eq!(dynamic.body, b"payload");

        let fallback = dispatch_local_http_route(
            None,
            NetHttpResponseDescriptor::new(NetRequestId::new(0), 202, b"queued".to_vec()),
            NetHttpRequestDescriptor::new(
                NetRequestId::new(42),
                NetHttpMethod::Get,
                "http://127.0.0.1/status",
            ),
        );
        assert_eq!(fallback.request, NetRequestId::new(42));
        assert_eq!(fallback.status_code, 202);
        assert_eq!(fallback.body, b"queued");
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn moved_local_http_request_release_benchmark_evidence() {
        let requests = benchmark_requests();
        let handler = Arc::new(|request: NetHttpRequestDescriptor| {
            NetHttpResponseDescriptor::new(request.request, 204, Vec::new())
        }) as HttpRouteHandler;
        let fallback = NetHttpResponseDescriptor::new(NetRequestId::new(0), 500, Vec::new());
        assert_eq!(
            legacy_dispatch_batch(requests.clone(), &handler, &fallback),
            moved_dispatch_batch(requests.clone(), &handler, &fallback)
        );

        let (legacy_samples, optimized_samples) =
            benchmark_paired_samples(&requests, &handler, &fallback);
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_raw_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_raw_ns = benchmark_samples_csv(&optimized_samples);
        let legacy_body_copy_bytes = BENCHMARK_BODY_BYTES * BENCHMARK_REQUEST_COUNT;

        println!(
            "PERF_RESULT task=plugins10_moved_local_http_request requests={BENCHMARK_REQUEST_COUNT} body_bytes_per_request={BENCHMARK_BODY_BYTES} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_request_clones_per_sample={BENCHMARK_REQUEST_COUNT} optimized_request_clones_per_sample=0 legacy_body_copy_bytes_per_sample={legacy_body_copy_bytes} optimized_body_copy_bytes_per_sample=0 threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_raw_ns} optimized_raw_ns={optimized_raw_ns}"
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_requests() -> Vec<NetHttpRequestDescriptor> {
        (1..=BENCHMARK_REQUEST_COUNT as u64)
            .map(|raw| {
                NetHttpRequestDescriptor::new(
                    NetRequestId::new(raw),
                    NetHttpMethod::Post,
                    "http://127.0.0.1/benchmark",
                )
                .with_body(vec![raw as u8; BENCHMARK_BODY_BYTES])
            })
            .collect()
    }

    fn legacy_dispatch_batch(
        requests: Vec<NetHttpRequestDescriptor>,
        handler: &HttpRouteHandler,
        fallback: &NetHttpResponseDescriptor,
    ) -> usize {
        let mut checksum = 0;
        for request in requests {
            let request_id = request.request;
            let local_handler = Some(Arc::clone(handler));
            let response = fallback.clone();
            let response = local_handler
                .map(|handler| handler(request.clone()))
                .unwrap_or_else(|| response.for_request(request_id));
            checksum += black_box(response.status_code as usize + response.body_bytes);
        }
        black_box(checksum)
    }

    fn moved_dispatch_batch(
        requests: Vec<NetHttpRequestDescriptor>,
        handler: &HttpRouteHandler,
        fallback: &NetHttpResponseDescriptor,
    ) -> usize {
        let mut checksum = 0;
        for request in requests {
            let response =
                dispatch_local_http_route(Some(Arc::clone(handler)), fallback.clone(), request);
            checksum += black_box(response.status_code as usize + response.body_bytes);
        }
        black_box(checksum)
    }

    fn benchmark_paired_samples(
        requests: &[NetHttpRequestDescriptor],
        handler: &HttpRouteHandler,
        fallback: &NetHttpResponseDescriptor,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(benchmark_sample(
            requests,
            handler,
            fallback,
            legacy_dispatch_batch,
        ));
        black_box(benchmark_sample(
            requests,
            handler,
            fallback,
            moved_dispatch_batch,
        ));
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(
                    requests,
                    handler,
                    fallback,
                    legacy_dispatch_batch,
                ));
                optimized_samples.push(benchmark_sample(
                    requests,
                    handler,
                    fallback,
                    moved_dispatch_batch,
                ));
            } else {
                optimized_samples.push(benchmark_sample(
                    requests,
                    handler,
                    fallback,
                    moved_dispatch_batch,
                ));
                legacy_samples.push(benchmark_sample(
                    requests,
                    handler,
                    fallback,
                    legacy_dispatch_batch,
                ));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(
        requests: &[NetHttpRequestDescriptor],
        handler: &HttpRouteHandler,
        fallback: &NetHttpResponseDescriptor,
        dispatch: fn(
            Vec<NetHttpRequestDescriptor>,
            &HttpRouteHandler,
            &NetHttpResponseDescriptor,
        ) -> usize,
    ) -> u128 {
        let requests = requests.to_vec();
        let started = Instant::now();
        let checksum = black_box(dispatch(requests, handler, fallback));
        let elapsed = started.elapsed().as_nanos();
        black_box(checksum);
        elapsed
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }
}

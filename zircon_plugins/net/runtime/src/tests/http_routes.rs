use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use tokio::runtime::Runtime;
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetHttpMethod, NetHttpRequestDescriptor,
    NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetManager, NetRequestId, NetRouteId,
};

use crate::{DefaultNetManager, HttpRuntimeBackend, ManagedHttpListener, ManagedHttpRoute};

#[test]
fn net_runtime_dispatches_registered_http_route() {
    let net = DefaultNetManager::default();
    let route = net
        .register_http_route(
            NetHttpRouteDescriptor::new("/health", [NetHttpMethod::Get]),
            NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, b"ok".to_vec())
                .with_header("content-type", "text/plain"),
        )
        .unwrap();

    let response = net
        .send_http_request(NetHttpRequestDescriptor::new(
            NetRequestId::new(7),
            NetHttpMethod::Get,
            "http://127.0.0.1/health",
        ))
        .unwrap();

    assert_eq!(response.request, NetRequestId::new(7));
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"ok");
    assert_eq!(response.body_bytes, 2);
    assert_eq!(net.diagnostics().open_http_routes, 1);
    net.unregister_http_route(route).unwrap();
    assert_eq!(net.diagnostics().open_http_routes, 0);
    assert!(net
        .drain_events(8)
        .iter()
        .any(|event| matches!(event, NetEvent::HttpRouteUnregistered { route: removed } if *removed == route)));
}

#[test]
fn net_runtime_dispatches_dynamic_http_route_handler() {
    let net = DefaultNetManager::default();
    net.register_http_route_handler(
        NetHttpRouteDescriptor::new("/echo", [NetHttpMethod::Post]),
        |request| NetHttpResponseDescriptor::new(request.request, 201, request.body),
    )
    .unwrap();

    let response = net
        .send_http_request(
            NetHttpRequestDescriptor::new(
                NetRequestId::new(31),
                NetHttpMethod::Post,
                "http://127.0.0.1/echo",
            )
            .with_body(b"payload".to_vec()),
        )
        .unwrap();

    assert_eq!(response.request, NetRequestId::new(31));
    assert_eq!(response.status_code, 201);
    assert_eq!(response.body, b"payload");
}

#[test]
fn dynamic_http_handler_can_reenter_manager_without_route_registry_deadlock() {
    let net = DefaultNetManager::default();
    let callback_net = net.clone();
    let route = net
        .register_http_route_handler(
            NetHttpRouteDescriptor::new("/reentrant", [NetHttpMethod::Get]),
            move |request| {
                assert_eq!(callback_net.diagnostics().open_http_routes, 1);
                NetHttpResponseDescriptor::new(request.request, 200, b"reentered".to_vec())
            },
        )
        .unwrap();

    let caller = net.clone();
    let (completed, result) = mpsc::sync_channel(1);
    let request_thread = thread::spawn(move || {
        let response = caller.send_http_request(NetHttpRequestDescriptor::new(
            NetRequestId::new(32),
            NetHttpMethod::Get,
            "http://127.0.0.1/reentrant",
        ));
        let _ = completed.send(response);
    });
    let response = result
        .recv_timeout(Duration::from_secs(2))
        .expect("reentrant HTTP handler must not deadlock the route registry")
        .unwrap();
    request_thread.join().unwrap();
    assert_eq!(response.body, b"reentered");
    net.unregister_http_route(route).unwrap();
    net.shutdown_worker_result_for_tests().unwrap();
}

#[test]
fn route_handler_destructor_can_reenter_manager_after_registry_release() {
    let net = DefaultNetManager::default();
    let reentrant_drop = ReentrantRouteDrop(net.clone());
    let route = net
        .register_http_route_handler(
            NetHttpRouteDescriptor::new("/drop-reentrant", [NetHttpMethod::Get]),
            move |request| {
                let _ = &reentrant_drop;
                NetHttpResponseDescriptor::new(request.request, 200, Vec::new())
            },
        )
        .unwrap();

    let caller = net.clone();
    let (completed, result) = mpsc::sync_channel(1);
    let unregister_thread = thread::spawn(move || {
        let _ = completed.send(caller.unregister_http_route(route));
    });
    result
        .recv_timeout(Duration::from_secs(2))
        .expect("route handler destructor must run outside the route registry guard")
        .unwrap();
    unregister_thread.join().unwrap();
    net.shutdown_worker_result_for_tests().unwrap();
}

#[test]
fn http_listener_post_callback_poison_aborts_and_does_not_publish_or_register() {
    let manager_slot = Arc::new(Mutex::new(None));
    let abort_slot = Arc::new(Mutex::new(None));
    let backend = Arc::new(PoisoningHttpListenerBackend {
        manager: Arc::clone(&manager_slot),
        abort: Arc::clone(&abort_slot),
    });
    let net = DefaultNetManager::default().with_http_backend(backend);
    *manager_slot.lock().unwrap() = Some(net.clone());

    assert_eq!(
        net.listen_http(&NetEndpoint::new("127.0.0.1", 0))
            .unwrap_err(),
        NetError::SharedStatePoisoned {
            resource: "net.http_listeners".to_string(),
        }
    );
    let abort = abort_slot
        .lock()
        .unwrap()
        .clone()
        .expect("backend must expose the spawned listener task");
    for _ in 0..100 {
        if abort.is_finished() {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(
        abort.is_finished(),
        "failed post-commit must abort listener task"
    );
    assert_eq!(net.diagnostics().open_http_listeners, 0);
    assert!(net.drain_events(usize::MAX).is_empty());

    *manager_slot.lock().unwrap() = None;
    net.shutdown_worker_result_for_tests().unwrap();
}

struct ReentrantRouteDrop(DefaultNetManager);

impl Drop for ReentrantRouteDrop {
    fn drop(&mut self) {
        let _ = self.0.diagnostics();
    }
}

#[derive(Debug)]
struct PoisoningHttpListenerBackend {
    manager: Arc<Mutex<Option<DefaultNetManager>>>,
    abort: Arc<Mutex<Option<tokio::task::AbortHandle>>>,
}

impl HttpRuntimeBackend for PoisoningHttpListenerBackend {
    fn listen_http(
        &self,
        runtime: &Runtime,
        bind: SocketAddr,
        _routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
    ) -> Result<ManagedHttpListener, NetError> {
        let manager = self
            .manager
            .lock()
            .unwrap()
            .as_ref()
            .expect("test manager must be installed")
            .clone();
        manager.poison_http_listeners_for_test();
        let abort = runtime.spawn(std::future::pending::<()>()).abort_handle();
        *self.abort.lock().unwrap() = Some(abort.clone());
        Ok(ManagedHttpListener {
            local_endpoint: NetEndpoint::from(bind),
            abort_handle: Some(abort),
        })
    }

    fn send_http_request(
        &self,
        _runtime: &Runtime,
        _request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError> {
        Err(NetError::ProtocolUnavailable {
            capability: "test.http.request".to_string(),
        })
    }
}

#[test]
fn base_net_runtime_requires_http_feature_for_real_socket_backend() {
    let net = DefaultNetManager::default();

    assert_eq!(
        net.listen_http(&NetEndpoint::new("127.0.0.1", 0))
            .unwrap_err(),
        NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.http".to_string(),
        }
    );
    assert_eq!(
        net.send_http_request(NetHttpRequestDescriptor::new(
            NetRequestId::new(17),
            NetHttpMethod::Get,
            "http://127.0.0.1:9/socket-health",
        ))
        .unwrap_err(),
        NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.http".to_string(),
        }
    );
}

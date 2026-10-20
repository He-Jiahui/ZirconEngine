use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_runtime::asset::pack::{ZrChunkEntry, ZrPackManifest, ZRPACK_FORMAT_VERSION};
use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadStatus, NetEndpoint,
    NetHttpMethod, NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetManager, NetRequestId,
};

use crate::NetContentDownloadRuntimeManager;

use super::support::zrpack_hash;

#[test]
fn content_download_manager_fetches_http_chunk_and_marks_complete_after_hash_match() {
    let http = zircon_plugin_net_http_runtime::http_runtime_manager();
    http.register_http_route(
        NetHttpRouteDescriptor::new("/chunks/full", [NetHttpMethod::Get]),
        NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, b"chunk-bytes".to_vec()),
    )
    .unwrap();
    let listener = http.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = http.listener_endpoint(listener).unwrap();

    let manager = NetContentDownloadRuntimeManager::with_net_manager(Arc::new(http.clone()));
    let download = NetDownloadId::new(12);
    let manifest = NetDownloadManifest::new(download, "asset://download/full").with_chunk(
        NetDownloadChunk::new(
            "chunk-full",
            format!("http://{}:{}/chunks/full", endpoint.host, endpoint.port),
            0,
            11,
            zrpack_hash(b"chunk-bytes"),
        ),
    );

    manager.queue_manifest(manifest);
    let fetched = manager.fetch_next_chunk(download, "chunk-full").unwrap();

    assert_eq!(fetched.status, NetDownloadStatus::Complete);
    assert_eq!(fetched.downloaded_bytes, 11);
    assert_eq!(fetched.completed_chunks, vec!["chunk-full".to_string()]);
}

#[test]
fn content_download_manager_fetches_resumed_http_range_with_existing_prefix() {
    let http = zircon_plugin_net_http_runtime::http_runtime_manager();
    let saw_range = Arc::new(AtomicBool::new(false));
    let saw_range_for_handler = saw_range.clone();
    http.register_http_route_handler(
        NetHttpRouteDescriptor::new("/chunks/range", [NetHttpMethod::Get]),
        move |request| {
            saw_range_for_handler.store(
                request.headers.iter().any(|(name, value)| {
                    name.eq_ignore_ascii_case("range") && value == "bytes=6-11"
                }),
                Ordering::SeqCst,
            );
            NetHttpResponseDescriptor::new(request.request, 206, b"suffix".to_vec())
                .with_header("content-range", "bytes 6-11/12")
        },
    )
    .unwrap();
    let listener = http.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = http.listener_endpoint(listener).unwrap();

    let manager = NetContentDownloadRuntimeManager::with_net_manager(Arc::new(http.clone()));
    let download = NetDownloadId::new(13);
    let manifest = NetDownloadManifest::new(download, "asset://download/range").with_chunk(
        NetDownloadChunk::new(
            "chunk-range",
            format!("http://{}:{}/chunks/range", endpoint.host, endpoint.port),
            0,
            12,
            zrpack_hash(b"prefixsuffix"),
        )
        .with_resume_from_byte(6),
    );

    manager.queue_manifest(manifest);
    manager.store_partial_chunk(download, "chunk-range", b"prefix".to_vec());
    let fetched = manager.fetch_next_chunk(download, "chunk-range").unwrap();

    assert_eq!(fetched.status, NetDownloadStatus::Complete);
    assert_eq!(fetched.downloaded_bytes, 12);
    assert!(saw_range.load(Ordering::SeqCst));
}

#[test]
fn content_download_manager_preserves_partial_prefix_after_corrupt_resume() {
    let http = zircon_plugin_net_http_runtime::http_runtime_manager();
    http.register_http_route_handler(
        NetHttpRouteDescriptor::new("/chunks/corrupt", [NetHttpMethod::Get]),
        move |request| {
            NetHttpResponseDescriptor::new(request.request, 206, b"corrupt".to_vec())
                .with_header("content-range", "bytes 6-12/13")
        },
    )
    .unwrap();
    let listener = http.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = http.listener_endpoint(listener).unwrap();

    let manager = NetContentDownloadRuntimeManager::with_net_manager(Arc::new(http.clone()));
    let download = NetDownloadId::new(16);
    let manifest = NetDownloadManifest::new(download, "asset://download/corrupt").with_chunk(
        NetDownloadChunk::new(
            "chunk-corrupt",
            format!("http://{}:{}/chunks/corrupt", endpoint.host, endpoint.port),
            0,
            13,
            zrpack_hash(b"prefixsuffix!"),
        )
        .with_resume_from_byte(6),
    );

    manager.queue_manifest(manifest);
    manager.store_partial_chunk(download, "chunk-corrupt", b"prefix".to_vec());
    let failed = manager.fetch_next_chunk(download, "chunk-corrupt").unwrap();

    assert_eq!(failed.status, NetDownloadStatus::Failed);
    assert_eq!(
        failed.diagnostic.as_deref(),
        Some("chunk hash mismatch: chunk-corrupt")
    );
    assert_eq!(
        manager.partial_chunk_bytes(download, "chunk-corrupt"),
        b"prefix"
    );
}

#[test]
fn corrupt_chunk_refetched() {
    let http = zircon_plugin_net_http_runtime::http_runtime_manager();
    let primary_hits = Arc::new(AtomicUsize::new(0));
    let primary_hits_for_handler = primary_hits.clone();
    http.register_http_route_handler(
        NetHttpRouteDescriptor::new("/chunks/refetch", [NetHttpMethod::Get]),
        move |request| {
            primary_hits_for_handler.fetch_add(1, Ordering::SeqCst);
            NetHttpResponseDescriptor::new(request.request, 200, b"corrupt".to_vec())
        },
    )
    .unwrap();
    let mirror_hits = Arc::new(AtomicUsize::new(0));
    let mirror_hits_for_handler = mirror_hits.clone();
    http.register_http_route_handler(
        NetHttpRouteDescriptor::new("/mirror/chunk-refetch", [NetHttpMethod::Get]),
        move |request| {
            mirror_hits_for_handler.fetch_add(1, Ordering::SeqCst);
            NetHttpResponseDescriptor::new(request.request, 200, b"correct".to_vec())
        },
    )
    .unwrap();
    let listener = http.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = http.listener_endpoint(listener).unwrap();

    let manager = NetContentDownloadRuntimeManager::with_net_manager(Arc::new(http.clone()));
    let download = NetDownloadId::new(18);
    let zrpack_manifest = ZrPackManifest::new(ZRPACK_FORMAT_VERSION, 7)
        .with_chunk(ZrChunkEntry::new(zrpack_hash(b"correct"), 0, 7));
    let manifest = NetDownloadManifest::new(download, "asset://download/refetch")
        .with_chunk(NetDownloadChunk::new(
            "chunk-refetch",
            format!("http://{}:{}/chunks/refetch", endpoint.host, endpoint.port),
            0,
            7,
            zrpack_manifest.chunks[0].hash,
        ))
        .with_mirror_url(format!("http://{}:{}/mirror", endpoint.host, endpoint.port));

    manager.queue_manifest(manifest);
    let failed_primary = manager.fetch_next_chunk(download, "chunk-refetch").unwrap();
    assert_eq!(failed_primary.status, NetDownloadStatus::Downloading);
    assert_eq!(
        failed_primary.diagnostic.as_deref(),
        Some("chunk attempt failed, switching mirror: chunk-refetch")
    );
    assert_eq!(
        manager.failed_attempts(download, "chunk-refetch"),
        vec!["chunk hash mismatch: chunk-refetch".to_string()]
    );

    let fetched_mirror = manager.fetch_next_chunk(download, "chunk-refetch").unwrap();
    assert_eq!(fetched_mirror.status, NetDownloadStatus::Complete);
    assert_eq!(
        fetched_mirror.completed_chunks,
        vec!["chunk-refetch".to_string()]
    );
    assert_eq!(primary_hits.load(Ordering::SeqCst), 1);
    assert_eq!(mirror_hits.load(Ordering::SeqCst), 1);
}

#[test]
fn content_download_manager_preserves_cancellation_after_in_flight_http_success() {
    use std::sync::mpsc;
    use std::time::Duration;

    let http = zircon_plugin_net_http_runtime::http_runtime_manager();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let release_rx = std::sync::Mutex::new(release_rx);
    http.register_http_route_handler(
        NetHttpRouteDescriptor::new("/chunks/cancel-inflight", [NetHttpMethod::Get]),
        move |request| {
            let _ = entered_tx.try_send(());
            let _ = release_rx
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5));
            NetHttpResponseDescriptor::new(request.request, 200, b"chunk-bytes".to_vec())
        },
    )
    .unwrap();
    let listener = http.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = http.listener_endpoint(listener).unwrap();
    let manager = NetContentDownloadRuntimeManager::with_net_manager(Arc::new(http.clone()));
    let download = NetDownloadId::new(286);
    manager.queue_manifest(
        NetDownloadManifest::new(download, "asset://download/cancel-inflight").with_chunk(
            NetDownloadChunk::new(
                "chunk",
                format!(
                    "http://{}:{}/chunks/cancel-inflight",
                    endpoint.host, endpoint.port
                ),
                0,
                11,
                zrpack_hash(b"chunk-bytes"),
            ),
        ),
    );
    manager.store_resume_bitmap(download, [false]);

    // Release the handler and join the real fetch before asserting, including
    // failed rendezvous paths. The handler itself also has a bounded wait.
    let (entered, cancelled, released, fetched) = std::thread::scope(|scope| {
        let fetch_manager = manager.clone();
        let fetch = scope.spawn(move || fetch_manager.fetch_next_chunk(download, "chunk"));
        let entered = entered_rx.recv_timeout(Duration::from_secs(5));
        let cancelled = manager.cancel_download(download);
        let released = release_tx.send(());
        (entered, cancelled, released, fetch.join())
    });
    let closed = http.close_listener(listener);
    assert!(
        entered.is_ok(),
        "the real HTTP request must reach its handler"
    );
    assert!(released.is_ok());
    closed.unwrap();
    let cancelled = cancelled.unwrap();
    assert_eq!(cancelled.status, NetDownloadStatus::Cancelled);
    assert_eq!(fetched.unwrap(), Some(cancelled.clone()));
    assert_eq!(manager.progress(download), Some(cancelled.clone()));
    assert_eq!(cancelled.diagnostic.as_deref(), Some("download cancelled"));
    assert_eq!(cancelled.downloaded_bytes, 0);
    assert!(cancelled.completed_chunks.is_empty());
    assert_eq!(manager.resume_bitmap(download), vec![false]);
    assert_eq!(
        manager.mark_cache_hit(download, "chunk"),
        Some(cancelled.clone())
    );
    assert_eq!(
        manager.mark_attempt_failed(download, "chunk", "late HTTP failure"),
        Some(cancelled.clone())
    );
    assert!(manager.failed_attempts(download, "chunk").is_empty());
    assert!(manager.cache_hits(download).is_empty());
    assert_eq!(manager.resume_bitmap(download), vec![false]);
    assert!(manager.next_attempt(download, "chunk").is_none());
    assert_eq!(manager.fetch_next_chunk(download, "chunk"), None);
    assert_eq!(manager.progress(download), Some(cancelled));
}

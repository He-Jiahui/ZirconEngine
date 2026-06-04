use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadStatus, NetEndpoint,
    NetHttpMethod, NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetManager, NetRequestId,
};

use crate::NetContentDownloadRuntimeManager;

use super::support::sha256_hex;

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
            sha256_hex(b"chunk-bytes"),
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
            sha256_hex(b"prefixsuffix"),
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
            sha256_hex(b"prefixsuffix!"),
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

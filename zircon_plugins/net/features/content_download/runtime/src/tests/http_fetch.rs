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

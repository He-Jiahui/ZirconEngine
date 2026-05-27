use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadStatus, NetEndpoint,
    NetHttpMethod, NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetManager, NetRequestId,
};

use super::{
    net_content_download_runtime_manager, plugin_feature_registration,
    NetContentDownloadRuntimeManager, NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY,
    NET_CONTENT_DOWNLOAD_FEATURE_ID, NET_CONTENT_DOWNLOAD_FEATURE_MANAGER_NAME,
    NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME,
};

#[test]
fn content_download_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_CONTENT_DOWNLOAD_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY));
    assert!(report.manifest.dependencies.iter().any(|dependency| {
        dependency.plugin_id == zircon_plugin_net_runtime::PLUGIN_ID
            && dependency.capability == "runtime.feature.net.http"
    }));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME)
        .expect("content download feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_CONTENT_DOWNLOAD_FEATURE_MANAGER_NAME
    );
}

#[test]
fn content_download_manager_tracks_chunk_progress_and_hash_failures() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(5);
    let manifest = NetDownloadManifest::new(download, "asset://texture/sky")
        .with_chunk(NetDownloadChunk::new(
            "chunk-a",
            "https://cdn.example/chunk-a",
            0,
            4,
            "hash-a",
        ))
        .with_chunk(NetDownloadChunk::new(
            "chunk-b",
            "https://cdn.example/chunk-b",
            4,
            4,
            "hash-b",
        ))
        .with_mirror_url("https://mirror.example/content");

    let queued = manager.queue_manifest(manifest);
    assert_eq!(queued.status, NetDownloadStatus::Queued);
    assert_eq!(queued.total_bytes, 8);
    assert_eq!(
        manager.candidate_urls(download, "chunk-a").unwrap(),
        vec![
            "https://cdn.example/chunk-a".to_string(),
            "https://mirror.example/content/chunk-a".to_string(),
        ]
    );

    let progress = manager
        .mark_chunk_complete(download, "chunk-a", "hash-a")
        .unwrap();
    assert_eq!(progress.status, NetDownloadStatus::Downloading);
    assert_eq!(progress.downloaded_bytes, 4);

    let failed = manager
        .mark_chunk_complete(download, "chunk-b", "wrong-hash")
        .unwrap();
    assert_eq!(failed.status, NetDownloadStatus::Failed);
    assert_eq!(
        failed.diagnostic.as_deref(),
        Some("chunk hash mismatch: chunk-b")
    );
}

#[test]
fn content_download_manager_records_cache_hits_as_completed_chunks() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(6);
    let manifest =
        NetDownloadManifest::new(download, "asset://mesh/tree").with_chunk(NetDownloadChunk::new(
            "chunk-cache",
            "https://cdn.example/cache",
            0,
            12,
            "hash-cache",
        ));

    manager.queue_manifest(manifest);
    let progress = manager.mark_cache_hit(download, "chunk-cache").unwrap();

    assert_eq!(progress.status, NetDownloadStatus::Complete);
    assert_eq!(progress.downloaded_bytes, 12);
    assert_eq!(
        manager.cache_hits(download),
        vec!["chunk-cache".to_string()]
    );
}

#[test]
fn content_download_manager_builds_range_resume_attempts_and_fails_over_mirrors() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(7);
    let manifest = NetDownloadManifest::new(download, "asset://patch/main")
        .with_chunk(
            NetDownloadChunk::new(
                "chunk-resume",
                "https://cdn.example/chunk-resume",
                256,
                1024,
                "hash-resume",
            )
            .with_resume_from_byte(512),
        )
        .with_mirror_url("https://mirror-a.example/content")
        .with_mirror_url("https://mirror-b.example/content");

    manager.queue_manifest(manifest);

    let first = manager.next_attempt(download, "chunk-resume").unwrap();
    assert_eq!(first.url, "https://cdn.example/chunk-resume");
    assert_eq!(first.byte_offset, 256);
    assert_eq!(first.byte_len, 1024);
    assert_eq!(first.range_start, Some(512));
    assert_eq!(first.attempt_index, 0);

    let switched = manager
        .mark_attempt_failed(download, "chunk-resume", "primary timeout")
        .unwrap();
    assert_eq!(switched.status, NetDownloadStatus::Downloading);
    assert_eq!(
        switched.diagnostic.as_deref(),
        Some("chunk attempt failed, switching mirror: chunk-resume")
    );
    let second = manager.next_attempt(download, "chunk-resume").unwrap();
    assert_eq!(second.url, "https://mirror-a.example/content/chunk-resume");
    assert_eq!(second.range_start, Some(512));
    assert_eq!(second.attempt_index, 1);

    manager.mark_attempt_failed(download, "chunk-resume", "mirror-a 503");
    let third = manager.next_attempt(download, "chunk-resume").unwrap();
    assert_eq!(third.url, "https://mirror-b.example/content/chunk-resume");
    assert_eq!(third.attempt_index, 2);

    let failed = manager
        .mark_attempt_failed(download, "chunk-resume", "mirror-b offline")
        .unwrap();
    assert_eq!(failed.status, NetDownloadStatus::Failed);
    assert_eq!(
        failed.diagnostic.as_deref(),
        Some("chunk attempts exhausted: chunk-resume")
    );
    assert!(manager.next_attempt(download, "chunk-resume").is_none());
    assert_eq!(
        manager.failed_attempts(download, "chunk-resume"),
        vec![
            "primary timeout".to_string(),
            "mirror-a 503".to_string(),
            "mirror-b offline".to_string(),
        ]
    );
}

#[test]
fn content_download_manager_can_cancel_queued_downloads() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(8);
    let manifest = NetDownloadManifest::new(download, "asset://cancel/me").with_chunk(
        NetDownloadChunk::new("chunk-cancel", "https://cdn.example/cancel", 0, 1, "hash"),
    );

    manager.queue_manifest(manifest);
    let cancelled = manager.cancel_download(download).unwrap();

    assert_eq!(cancelled.status, NetDownloadStatus::Cancelled);
    assert_eq!(cancelled.diagnostic.as_deref(), Some("download cancelled"));
}

#[test]
fn content_download_manager_rejects_empty_manifest_before_queueing() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(9);
    let manifest = NetDownloadManifest::new(download, "asset://empty/package");

    let rejected = manager.queue_manifest(manifest);

    assert_eq!(rejected.status, NetDownloadStatus::Failed);
    assert_eq!(rejected.total_bytes, 0);
    assert_eq!(
        rejected.diagnostic.as_deref(),
        Some("download manifest has no chunks")
    );
    assert!(manager.progress(download).is_none());
    assert!(manager.candidate_urls(download, "missing").is_none());
}

#[test]
fn content_download_manager_rejects_duplicate_chunk_ids_before_queueing() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(10);
    let manifest = NetDownloadManifest::new(download, "asset://duplicate/package")
        .with_chunk(NetDownloadChunk::new(
            "chunk-duplicate",
            "https://cdn.example/a",
            0,
            4,
            "hash-a",
        ))
        .with_chunk(NetDownloadChunk::new(
            "chunk-duplicate",
            "https://cdn.example/b",
            4,
            4,
            "hash-b",
        ));

    let rejected = manager.queue_manifest(manifest);

    assert_eq!(rejected.status, NetDownloadStatus::Failed);
    assert_eq!(
        rejected.diagnostic.as_deref(),
        Some("duplicate download chunk id: chunk-duplicate")
    );
    assert!(manager.progress(download).is_none());
}

#[test]
fn content_download_manager_rejects_invalid_chunk_fields_before_queueing() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(11);
    let manifest = NetDownloadManifest::new(download, "asset://invalid/package").with_chunk(
        NetDownloadChunk::new("chunk-invalid", "", 0, 0, "hash-invalid"),
    );

    let rejected = manager.queue_manifest(manifest);

    assert_eq!(rejected.status, NetDownloadStatus::Failed);
    assert_eq!(
        rejected.diagnostic.as_deref(),
        Some("download chunk has empty URL: chunk-invalid")
    );
    assert!(manager.progress(download).is_none());
}

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
fn content_download_manager_fails_resumed_http_range_without_existing_prefix() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(14);
    let manifest = NetDownloadManifest::new(download, "asset://download/no-prefix").with_chunk(
        NetDownloadChunk::new(
            "chunk-no-prefix",
            "http://127.0.0.1:9/missing",
            0,
            8,
            "hash",
        )
        .with_resume_from_byte(4),
    );

    manager.queue_manifest(manifest);
    let failed = manager
        .fetch_next_chunk(download, "chunk-no-prefix")
        .unwrap();

    assert_eq!(failed.status, NetDownloadStatus::Failed);
    assert_eq!(
        failed.diagnostic.as_deref(),
        Some("chunk resume requires existing partial bytes: chunk-no-prefix")
    );
}

#[test]
fn content_download_manager_rejects_resume_offsets_before_chunk_start() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(15);
    let manifest = NetDownloadManifest::new(download, "asset://download/bad-resume").with_chunk(
        NetDownloadChunk::new(
            "chunk-bad-resume",
            "https://cdn.example/chunk",
            4,
            8,
            "hash",
        )
        .with_resume_from_byte(3),
    );

    let rejected = manager.queue_manifest(manifest);

    assert_eq!(rejected.status, NetDownloadStatus::Failed);
    assert_eq!(
        rejected.diagnostic.as_deref(),
        Some("download chunk resume offset outside range: chunk-bad-resume")
    );
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

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, bytes);
    digest
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

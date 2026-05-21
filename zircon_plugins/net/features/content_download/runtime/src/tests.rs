use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadStatus,
};

use super::{
    net_content_download_runtime_manager, plugin_feature_registration,
    NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY, NET_CONTENT_DOWNLOAD_FEATURE_ID,
    NET_CONTENT_DOWNLOAD_FEATURE_MANAGER_NAME, NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME,
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

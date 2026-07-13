use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadStatus,
};

use crate::net_content_download_runtime_manager;

use super::support::zrpack_hash;

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
            zrpack_hash(b"hash-a"),
        ))
        .with_chunk(NetDownloadChunk::new(
            "chunk-b",
            "https://cdn.example/chunk-b",
            4,
            4,
            zrpack_hash(b"hash-b"),
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
        .mark_chunk_complete(download, "chunk-a", &zrpack_hash(b"hash-a"))
        .unwrap();
    assert_eq!(progress.status, NetDownloadStatus::Downloading);
    assert_eq!(progress.downloaded_bytes, 4);

    let failed = manager
        .mark_chunk_complete(download, "chunk-b", &zrpack_hash(b"wrong-hash"))
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
            zrpack_hash(b"hash-cache"),
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
fn content_download_manager_can_cancel_queued_downloads() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(8);
    let manifest =
        NetDownloadManifest::new(download, "asset://cancel/me").with_chunk(NetDownloadChunk::new(
            "chunk-cancel",
            "https://cdn.example/cancel",
            0,
            1,
            zrpack_hash(b"hash"),
        ));

    manager.queue_manifest(manifest);
    let cancelled = manager.cancel_download(download).unwrap();

    assert_eq!(cancelled.status, NetDownloadStatus::Cancelled);
    assert_eq!(cancelled.diagnostic.as_deref(), Some("download cancelled"));
}

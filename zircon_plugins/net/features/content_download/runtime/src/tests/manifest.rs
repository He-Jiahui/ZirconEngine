use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadStatus,
};

use crate::net_content_download_runtime_manager;

use super::support::zrpack_hash;

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
            zrpack_hash(b"hash-a"),
        ))
        .with_chunk(NetDownloadChunk::new(
            "chunk-duplicate",
            "https://cdn.example/b",
            4,
            4,
            zrpack_hash(b"hash-b"),
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
        NetDownloadChunk::new("chunk-invalid", "", 0, 0, zrpack_hash(b"hash-invalid")),
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
fn content_download_manager_rejects_resume_offsets_before_chunk_start() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(15);
    let manifest = NetDownloadManifest::new(download, "asset://download/bad-resume").with_chunk(
        NetDownloadChunk::new(
            "chunk-bad-resume",
            "https://cdn.example/chunk",
            4,
            8,
            zrpack_hash(b"hash"),
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
fn content_download_manager_rejects_total_byte_overflow_before_queueing() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(19);
    let manifest = NetDownloadManifest::new(download, "asset://download/total-overflow")
        .with_chunk(NetDownloadChunk::new(
            "chunk-large",
            "https://cdn.example/chunk-large",
            0,
            u64::MAX,
            zrpack_hash(b"large"),
        ))
        .with_chunk(NetDownloadChunk::new(
            "chunk-tail",
            "https://cdn.example/chunk-tail",
            0,
            1,
            zrpack_hash(b"tail"),
        ));

    let rejected = manager.queue_manifest(manifest);

    assert_eq!(rejected.status, NetDownloadStatus::Failed);
    assert_eq!(rejected.total_bytes, 0);
    assert_eq!(
        rejected.diagnostic.as_deref(),
        Some("download manifest total byte size overflow")
    );
    assert!(manager.progress(download).is_none());
}

#[test]
fn content_download_manager_rejects_chunk_range_overflow_before_queueing() {
    let manager = net_content_download_runtime_manager();
    let download = NetDownloadId::new(20);
    let manifest = NetDownloadManifest::new(download, "asset://download/range-overflow")
        .with_chunk(
            NetDownloadChunk::new(
                "chunk-overflow",
                "https://cdn.example/chunk-overflow",
                u64::MAX - 1,
                4,
                zrpack_hash(b"overflow"),
            )
            .with_resume_from_byte(u64::MAX),
        );

    let rejected = manager.queue_manifest(manifest);

    assert_eq!(rejected.status, NetDownloadStatus::Failed);
    assert_eq!(rejected.total_bytes, 4);
    assert_eq!(
        rejected.diagnostic.as_deref(),
        Some("download chunk byte range overflow: chunk-overflow")
    );
    assert!(manager.progress(download).is_none());
}

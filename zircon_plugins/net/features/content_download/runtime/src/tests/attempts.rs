use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadStatus,
};

use crate::net_content_download_runtime_manager;

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

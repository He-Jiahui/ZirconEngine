use zircon_runtime::core::framework::net::{
    NetDownloadChunk, NetDownloadId, NetDownloadManifest, NetDownloadStatus,
};

use crate::net_content_download_runtime_manager;

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

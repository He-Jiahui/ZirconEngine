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
        ));

    let queued = manager.queue_manifest(manifest);
    assert_eq!(queued.status, NetDownloadStatus::Queued);
    assert_eq!(queued.total_bytes, 8);

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

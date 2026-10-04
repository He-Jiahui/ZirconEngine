use std::fs;

use super::{
    decode_pipeline_cache_seed, encode_pipeline_cache_seed, pipeline_cache_gate,
    pipeline_cache_key_from_ids, read_pipeline_cache_seed_with_limit, PipelineCacheGate,
};
use zr_rhi::RenderBackendKind;

#[test]
fn render_perf_pipeline_cache_gate_is_vulkan_only_and_reports_cold_seed() {
    assert_eq!(
        pipeline_cache_gate(RenderBackendKind::Vulkan, true, true),
        PipelineCacheGate::Enabled
    );
    assert_eq!(
        pipeline_cache_gate(RenderBackendKind::Vulkan, false, true),
        PipelineCacheGate::MissingSeedData
    );
    assert_eq!(
        pipeline_cache_gate(RenderBackendKind::Dx12, true, true),
        PipelineCacheGate::UnsupportedBackend
    );
    assert_eq!(
        pipeline_cache_gate(RenderBackendKind::Vulkan, true, false),
        PipelineCacheGate::UnsupportedDeviceFeature
    );
}

#[test]
fn render_perf_pipeline_cache_key_preserves_wgpu_vulkan_compatibility() {
    assert_eq!(
        pipeline_cache_key_from_ids(RenderBackendKind::Vulkan, 4_314, 8_675),
        Some("wgpu_pipeline_cache_vulkan_4314_8675".to_owned())
    );
    assert_eq!(
        pipeline_cache_key_from_ids(RenderBackendKind::Dx12, 4_314, 8_675),
        None
    );
}

#[test]
fn render_perf_pipeline_cache_seed_rejects_corruption_and_truncation() {
    let mut encoded = encode_pipeline_cache_seed(b"driver-cache-data");
    assert_eq!(
        decode_pipeline_cache_seed(&encoded),
        Some(b"driver-cache-data".to_vec())
    );

    let last = encoded.len() - 1;
    encoded[last] ^= 0x5a;
    assert_eq!(decode_pipeline_cache_seed(&encoded), None);
    assert_eq!(decode_pipeline_cache_seed(&encoded[..10]), None);
}

#[test]
fn render_perf_pipeline_cache_seed_read_is_bounded_without_second_decode_copy() {
    let root = std::env::temp_dir().join(format!(
        "zircon-pipeline-cache-read-bound-{}",
        std::process::id()
    ));
    let path = root.join("seed.bin");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(&path, encode_pipeline_cache_seed(b"12345")).unwrap();

    assert_eq!(
        read_pipeline_cache_seed_with_limit(&path, 5),
        Some(b"12345".to_vec())
    );
    assert_eq!(read_pipeline_cache_seed_with_limit(&path, 4), None);

    let _ = fs::remove_dir_all(root);
}

use super::*;

#[test]
fn render_shadow_atlas_resource_config_normalizes_zero_values() {
    let config = ShadowAtlasResourceConfig::new(0, 0, 0).normalized();

    assert_eq!(config.width, 1);
    assert_eq!(config.height, 1);
    assert_eq!(config.slot_capacity, 1);
    assert_eq!(
        config.slot_buffer_size_bytes(),
        GPU_SHADOW_SLOT_STRIDE as u64
    );
}

#[test]
fn render_shadow_atlas_resource_config_uses_plan_05_defaults() {
    let config = ShadowAtlasResourceConfig::default();

    assert_eq!(config.width, 4096);
    assert_eq!(config.height, 4096);
    assert_eq!(config.slot_capacity, SHADOW_ATLAS_DEFAULT_SLOT_CAPACITY);
    assert_eq!(
        config.slot_buffer_size_bytes(),
        SHADOW_ATLAS_DEFAULT_SLOT_CAPACITY as u64 * GPU_SHADOW_SLOT_STRIDE as u64
    );
}

#[test]
fn render_shadow_atlas_compare_function_matches_forward_depth_contract() {
    assert_eq!(
        SHADOW_ATLAS_COMPARE_FUNCTION,
        wgpu::CompareFunction::LessEqual
    );
}

#[test]
fn render_shadow_atlas_resource_config_downgrades_to_capability_limit() {
    let config = ShadowAtlasResourceConfig::new(4096, 4096, 16).with_max_texture_dimension(3072);

    assert_eq!(config.width, SHADOW_ATLAS_FALLBACK_SIZE);
    assert_eq!(config.height, SHADOW_ATLAS_FALLBACK_SIZE);
    assert_eq!(config.slot_capacity, 16);
}

#[test]
fn render_shadow_atlas_upload_report_describes_cleared_tail() {
    let report = ShadowAtlasUploadReport {
        uploaded_slot_count: 2,
        cleared_stale_slot_count: 3,
        slot_capacity: 8,
    };

    assert_eq!(report.uploaded_slot_count, 2);
    assert_eq!(report.cleared_stale_slot_count, 3);
    assert_eq!(report.slot_capacity, 8);
}

#[test]
fn shadow_frame_uploads_are_prepared_without_native_queue_writes() {
    let production = include_str!("../resources.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("shadow resource test boundary");

    assert!(production.contains("ShadowAtlasPreparedUpload"));
    assert!(production.contains("frame_batch.append(&mut self.batch)"));
    assert!(!production.contains("queue.write_buffer"));
}

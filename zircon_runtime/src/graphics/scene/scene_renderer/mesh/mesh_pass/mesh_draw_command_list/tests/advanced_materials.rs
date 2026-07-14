use super::*;

#[test]
fn render_advanced_static_material_bypasses_opaque_cache_and_keeps_late_forward_command() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let mut batch = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_cache_identity(7, 0)
        .with_static_state(RenderMeshStaticState::new(true, 11, 17));
    batch.pipeline_key.pbr_clearcoat = true;

    let buffers = build_mesh_pass_command_buffers_from_batches_cached(
        [batch],
        &mut variants,
        &mut cache,
        1,
        ShaderQualityTier::default(),
    );

    assert!(buffers.opaque().commands().is_empty());
    assert_eq!(buffers.advanced_pbr_opaque().commands().len(), 1);
    assert!(buffers.transparent().commands().is_empty());
    assert_eq!(buffers.stats().cached_command_hit_count, 0);
    assert!(buffers.stats().dynamic_command_count > 0);
}

#[test]
fn render_advanced_command_lists_keep_transmission_after_late_forward_opaque() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut clearcoat = batch(MeshDrawQueuePhase::Opaque, 10);
    clearcoat.pipeline_key.pbr_clearcoat = true;
    let mut transmission = batch(MeshDrawQueuePhase::Transparent, 20);
    transmission.pipeline_key.pbr_transmission = true;

    let buffers = build_mesh_pass_command_buffers_from_batches(
        [clearcoat, transmission],
        &mut variants,
        ShaderQualityTier::default(),
    );

    assert_eq!(buffers.advanced_pbr_opaque().commands().len(), 1);
    assert_eq!(buffers.transmission().commands().len(), 1);
    assert!(buffers.transparent().commands().is_empty());
    assert!(
        !buffers.advanced_pbr_opaque().commands()[0]
            .pipeline_key()
            .pbr_transmission
    );
    assert!(
        buffers.transmission().commands()[0]
            .pipeline_key()
            .pbr_transmission
    );
}

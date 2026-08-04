use super::*;
use crate::graphics::scene::resources::MaterialDisabledPasses;

#[test]
fn render_material_options_disabled_passes_and_queue_drive_mesh_commands_together() {
    let mut list = MeshDrawCommandList::new();
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut context = MeshPassBuildContext::with_default_quality(&mut variants);
    let default_material = batch(MeshDrawQueuePhase::Opaque, 1).with_casts_shadow(true);
    let mut coated_key = default_pipeline_key();
    coated_key.material_option_bits = 1;
    let coated_material = batch_with_pipeline_key(MeshDrawQueuePhase::Opaque, 2, coated_key)
        .with_casts_shadow(true)
        .with_disabled_passes(MaterialDisabledPasses::from_shader_pass_names(&[
            "shadow".to_string()
        ]));
    let transparent_material = batch(MeshDrawQueuePhase::Transparent, 3);

    OpaqueBasePassProcessor.add_mesh_batch(&default_material, &mut context, &mut list);
    OpaqueBasePassProcessor.add_mesh_batch(&coated_material, &mut context, &mut list);
    ShadowPassProcessor.add_mesh_batch(&default_material, &mut context, &mut list);
    ShadowPassProcessor.add_mesh_batch(&coated_material, &mut context, &mut list);
    TransparentPassProcessor.add_mesh_batch(&transparent_material, &mut context, &mut list);
    list.sort();

    assert_eq!(
        list.commands()
            .iter()
            .map(|command| command.phase)
            .collect::<Vec<_>>(),
        vec![
            RenderPhase::Shadow,
            RenderPhase::Opaque3d,
            RenderPhase::Opaque3d,
            RenderPhase::Transparent3d,
        ]
    );
    let option_bits = list
        .commands()
        .iter()
        .filter(|command| command.phase == RenderPhase::Opaque3d)
        .map(|command| {
            variants
                .key_for_variant(command.pipeline_variant_id)
                .expect("base command should retain its registered variant")
                .shader_variant_key()
                .material_option_bits
        })
        .collect::<Vec<_>>();
    assert_eq!(option_bits, vec![0, 1]);
    assert_eq!(variants.len(), 3);
    let miss_report = variants.miss_report();
    assert_eq!(miss_report.request_count, 4);
    assert_eq!(miss_report.memory_hit_count, 1);
    assert_eq!(miss_report.compile_miss_count, 0);
    assert_eq!(
        list.commands()
            .iter()
            .filter(|command| command.phase == RenderPhase::Shadow)
            .count(),
        1
    );
}

#[test]
fn render_advanced_material_lobes_route_opaque_and_masked_batches_to_late_forward() {
    let mut list = MeshDrawCommandList::new();
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut context = MeshPassBuildContext::with_default_quality(&mut variants);
    let mut clearcoat_key = default_pipeline_key();
    clearcoat_key.pbr_clearcoat = true;
    let mut anisotropy_key = default_pipeline_key();
    anisotropy_key.pbr_anisotropy = true;
    anisotropy_key.alpha_mask = true;

    let clearcoat = batch_with_pipeline_key(MeshDrawQueuePhase::Opaque, 20, clearcoat_key);
    let anisotropy = batch_with_pipeline_key(MeshDrawQueuePhase::AlphaMask, 21, anisotropy_key);

    OpaqueBasePassProcessor.add_mesh_batch(&clearcoat, &mut context, &mut list);
    OpaqueBasePassProcessor.add_mesh_batch(&anisotropy, &mut context, &mut list);
    list.sort();

    assert_eq!(list.commands().len(), 2);
    assert!(
        list.commands()
            .iter()
            .all(|command| command.phase == RenderPhase::Transparent3d)
    );
    assert!(list.commands().iter().all(|command| {
        command.pipeline_key().requires_forward_path()
            && command.pipeline_kind == MeshPassPipelineKind::Base
    }));
}

fn batch_with_pipeline_key(
    phase: MeshDrawQueuePhase,
    sort_key: u64,
    pipeline_key: PipelineKey,
) -> MeshBatchRef {
    batch_with_geometry_and_pipeline_key(
        phase,
        sort_key,
        MeshDrawGeometrySource::Prepared,
        pipeline_key,
    )
}

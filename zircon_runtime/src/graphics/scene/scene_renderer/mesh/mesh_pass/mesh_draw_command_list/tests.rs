use crate::core::framework::render::{
    RenderCapabilitySummary, RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents,
    ShaderQualityTier,
};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    CachedMeshDrawCommands, DrawInstanceSource, MeshBatchRef, MeshBindHandle, MeshDrawArgs,
    MeshDrawCommand, MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantRegistry;

use super::builder::{
    build_mesh_pass_command_buffers_from_batches,
    build_mesh_pass_command_buffers_from_batches_cached,
};
use super::{MeshDrawCommandList, MeshDrawCommandListStats, MeshPassCommandBufferStats};

#[test]
fn mesh_draw_command_list_sorts_by_phase_then_sort_key() {
    let commands = MeshDrawCommandList::from_commands(vec![
        command(RenderPhase::Transparent3d, 10, 3),
        command(RenderPhase::Opaque3d, 50, 1),
        command(RenderPhase::Opaque3d, 20, 2),
        command(RenderPhase::Shadow, 90, 4),
    ]);

    let ordered = commands
        .commands()
        .iter()
        .map(|command| (command.phase, command.sort_key))
        .collect::<Vec<_>>();

    assert_eq!(
        ordered,
        vec![
            (RenderPhase::Shadow, 90),
            (RenderPhase::Opaque3d, 20),
            (RenderPhase::Opaque3d, 50),
            (RenderPhase::Transparent3d, 10),
        ]
    );
}

#[test]
fn mesh_draw_command_list_reports_draw_and_instance_sources() {
    let commands = MeshDrawCommandList::from_commands(vec![
        command(RenderPhase::Opaque3d, 1, 1),
        MeshDrawCommand::new(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(2),
            2,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 7,
                instance_count: 1,
            },
            MeshGeometryHandle::test(2),
            MeshDrawArgs::test_indexed_indirect(9, 64),
        ),
    ]);

    assert_eq!(
        commands.stats(),
        MeshDrawCommandListStats {
            command_count: 2,
            direct_indexed_count: 1,
            indirect_indexed_count: 1,
            gpu_scene_instance_count: 2,
        }
    );
}

#[test]
fn mesh_batch_ref_emits_gpu_scene_instance_command() {
    let command = batch(MeshDrawQueuePhase::Opaque, 10)
        .with_cache_identity(42, 0)
        .with_gpu_scene_instance_span(7, 2)
        .command(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            MeshPipelineVariantId::new(4),
        );

    match &command.instance_source {
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index,
            instance_count,
        } => {
            assert_eq!(*first_instance_index, 7);
            assert_eq!(*instance_count, 2);
        }
    }
    match &command.draw_args {
        MeshDrawArgs::DirectIndexed {
            first_instance,
            instance_count,
            ..
        } => {
            assert_eq!(*first_instance, 7);
            assert_eq!(*instance_count, 2);
        }
        MeshDrawArgs::IndexedIndirect { .. } => panic!("expected direct indexed draw args"),
    }
    assert_eq!(command.source_entity, 42);
}

#[test]
fn mesh_draw_command_list_filters_phase_views_without_resorting() {
    let commands = MeshDrawCommandList::from_commands(vec![
        command(RenderPhase::Opaque3d, 20, 1),
        command(RenderPhase::Transparent3d, 10, 2),
        command(RenderPhase::Opaque3d, 30, 3),
    ]);

    let opaque_sort_keys = commands
        .iter_phase(RenderPhase::Opaque3d)
        .map(|command| command.sort_key)
        .collect::<Vec<_>>();

    assert_eq!(opaque_sort_keys, vec![20, 30]);
}

#[test]
fn mesh_pass_commands_sort_opaque_by_state_bucket_before_depth() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let buffers = build_mesh_pass_command_buffers_from_batches(
        [
            batch_with_depth(MeshDrawQueuePhase::Opaque, 10.0, 10).with_source_draw_index(0),
            batch_with_depth(MeshDrawQueuePhase::Opaque, 1.0, 20).with_source_draw_index(1),
        ],
        &mut variants,
        ShaderQualityTier::default(),
    );

    assert_eq!(
        buffers
            .opaque()
            .commands()
            .iter()
            .map(|command| command.source_draw_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
}

#[test]
fn mesh_pass_commands_sort_transparent_by_depth_before_pipeline_bucket() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let buffers = build_mesh_pass_command_buffers_from_batches(
        [
            batch_with_depth(MeshDrawQueuePhase::Transparent, 1.0, 10).with_source_draw_index(0),
            batch_with_depth(MeshDrawQueuePhase::Transparent, 100.0, 20).with_source_draw_index(1),
        ],
        &mut variants,
        ShaderQualityTier::default(),
    );

    assert_eq!(
        buffers
            .transparent()
            .commands()
            .iter()
            .map(|command| command.source_draw_index)
            .collect::<Vec<_>>(),
        vec![1, 0]
    );
}

#[test]
fn mesh_pass_command_buffers_build_expected_phase_counts_from_batches() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let buffers = build_mesh_pass_command_buffers_from_batches(
        [
            batch(MeshDrawQueuePhase::Opaque, 10)
                .with_source_draw_index(0)
                .with_casts_shadow(true)
                .with_previous_velocity_transform(true)
                .with_taa_reactive_mask_strength(0.75),
            batch(MeshDrawQueuePhase::AlphaMask, 20)
                .with_source_draw_index(1)
                .with_casts_shadow(true)
                .with_taa_reactive_mask_strength(0.5),
            batch(MeshDrawQueuePhase::Transparent, 30).with_source_draw_index(2),
            batch(MeshDrawQueuePhase::Opaque, 40).with_source_draw_index(3),
        ],
        &mut variants,
        ShaderQualityTier::default(),
    );

    assert_eq!(buffers.depth_prepass().commands().len(), 3);
    assert_eq!(buffers.shadow().commands().len(), 2);
    assert_eq!(buffers.opaque().commands().len(), 2);
    assert_eq!(buffers.alpha_mask().commands().len(), 1);
    assert_eq!(buffers.transparent().commands().len(), 1);
    assert_eq!(buffers.velocity().commands().len(), 1);
    assert_eq!(buffers.taa_reactive_mask().commands().len(), 3);
    assert_eq!(
        buffers
            .opaque()
            .commands()
            .iter()
            .map(|command| command.source_draw_index)
            .collect::<Vec<_>>(),
        vec![0, 3]
    );
    assert_eq!(
        buffers.stats(),
        MeshPassCommandBufferStats {
            command_count: 13,
            depth_prepass_command_count: 3,
            shadow_command_count: 2,
            opaque_command_count: 2,
            alpha_mask_command_count: 1,
            transparent_command_count: 1,
            velocity_command_count: 1,
            taa_reactive_mask_command_count: 3,
            direct_indexed_count: 13,
            indirect_indexed_count: 0,
            gpu_scene_instance_count: 13,
            cached_command_hit_count: 0,
            command_rebuild_count: 13,
            dynamic_command_count: 13,
            cache_miss_count: 0,
            cache_invalidated_transform_count: 0,
            cache_invalidated_geometry_count: 0,
            cache_invalidated_material_count: 0,
            indirect_batch_count: 0,
            indirect_batched_draw_count: 0,
            indirect_fallback_draw_count: 13,
            indirect_args_count: 0,
        }
    );
}

#[test]
fn mesh_pass_command_buffers_report_indirect_batch_stats_when_gpu_driven_supported() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let buffers = build_mesh_pass_command_buffers_from_batches(
        [
            batch(MeshDrawQueuePhase::Opaque, 10),
            batch(MeshDrawQueuePhase::Opaque, 10),
        ],
        &mut variants,
        ShaderQualityTier::default(),
    );

    let default_stats = buffers.stats();
    assert_eq!(default_stats.indirect_batch_count, 0);
    assert_eq!(default_stats.indirect_fallback_draw_count, 4);

    let stats = buffers.stats_with_indirect_batches(&gpu_driven_capabilities());

    assert_eq!(stats.command_count, 4);
    assert_eq!(stats.indirect_batch_count, 2);
    assert_eq!(stats.indirect_batched_draw_count, 4);
    assert_eq!(stats.indirect_fallback_draw_count, 0);
    assert_eq!(stats.indirect_args_count, 4);
}

#[test]
fn mesh_pass_command_buffers_assign_cache_variants_by_pipeline_kind() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let buffers = build_mesh_pass_command_buffers_from_batches(
        [batch(MeshDrawQueuePhase::Opaque, 10)
            .with_casts_shadow(true)
            .with_previous_velocity_transform(true)
            .with_taa_reactive_mask_strength(0.75)],
        &mut variants,
        ShaderQualityTier::default(),
    );

    let depth = buffers.depth_prepass().commands()[0].pipeline_variant_id;
    let shadow = buffers.shadow().commands()[0].pipeline_variant_id;
    let opaque = buffers.opaque().commands()[0].pipeline_variant_id;
    let velocity = buffers.velocity().commands()[0].pipeline_variant_id;
    let reactive = buffers.taa_reactive_mask().commands()[0].pipeline_variant_id;

    assert_eq!(depth, MeshPipelineVariantId::new(0));
    assert_eq!(shadow, MeshPipelineVariantId::new(0));
    assert_ne!(opaque, MeshPipelineVariantId::new(0));
    assert_ne!(velocity, MeshPipelineVariantId::new(0));
    assert_ne!(reactive, MeshPipelineVariantId::new(0));
    assert_ne!(opaque, velocity);
    assert_ne!(opaque, reactive);
    assert_ne!(velocity, reactive);
    assert_eq!(variants.len(), 3);
}

#[test]
fn mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let static_state = RenderMeshStaticState::new(true, 11, 17);
    let batch = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_cache_identity(7, 0)
        .with_static_state(static_state)
        .with_casts_shadow(true)
        .with_taa_reactive_mask_strength(0.75);

    let first = build_mesh_pass_command_buffers_from_batches_cached(
        [batch.clone()],
        &mut variants,
        &mut cache,
        1,
        ShaderQualityTier::default(),
    )
    .stats();
    let second = build_mesh_pass_command_buffers_from_batches_cached(
        [batch],
        &mut variants,
        &mut cache,
        2,
        ShaderQualityTier::default(),
    )
    .stats();

    assert_eq!(first.command_count, 4);
    assert_eq!(first.cached_command_hit_count, 0);
    assert_eq!(first.command_rebuild_count, 4);
    assert_eq!(first.dynamic_command_count, 1);
    assert_eq!(first.cache_miss_count, 3);
    assert_eq!(first.cache_invalidated_material_count, 0);
    assert_eq!(second.command_count, 4);
    assert_eq!(second.cached_command_hit_count, 3);
    assert_eq!(second.command_rebuild_count, 1);
    assert_eq!(second.dynamic_command_count, 1);
    assert_eq!(second.cache_miss_count, 0);
    assert_eq!(second.cache_invalidated_material_count, 0);
}

#[test]
fn mesh_pass_command_buffers_report_static_cache_invalidation_reasons() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let static_state = RenderMeshStaticState::new(true, 11, 17);
    let changed_material = RenderMeshStaticState::new(true, 11, 23);
    let batch = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_cache_identity(7, 0)
        .with_static_state(static_state)
        .with_casts_shadow(true);

    let _ = build_mesh_pass_command_buffers_from_batches_cached(
        [batch],
        &mut variants,
        &mut cache,
        1,
        ShaderQualityTier::default(),
    );
    let changed = build_mesh_pass_command_buffers_from_batches_cached(
        [static_batch(MeshDrawQueuePhase::Opaque, 10)
            .with_cache_identity(7, 0)
            .with_static_state(changed_material)
            .with_casts_shadow(true)],
        &mut variants,
        &mut cache,
        2,
        ShaderQualityTier::default(),
    )
    .stats();

    assert_eq!(changed.cached_command_hit_count, 0);
    assert_eq!(changed.cache_miss_count, 0);
    assert_eq!(changed.cache_invalidated_transform_count, 0);
    assert_eq!(changed.cache_invalidated_geometry_count, 0);
    assert_eq!(changed.cache_invalidated_material_count, 3);
    assert_eq!(changed.command_rebuild_count, 3);
}

fn command(phase: RenderPhase, sort_key: u64, variant_id: u32) -> MeshDrawCommand {
    MeshDrawCommand::new(
        phase,
        MeshPassPipelineKind::Base,
        default_pipeline_key(),
        MeshPipelineVariantId::new(variant_id),
        sort_key,
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index: variant_id,
            instance_count: 1,
        },
        MeshGeometryHandle::test(u64::from(variant_id)),
        MeshDrawArgs::direct_indexed(0, 3).with_instance_span(variant_id, 1),
    )
}

fn batch(phase: MeshDrawQueuePhase, sort_key: u64) -> MeshBatchRef {
    MeshBatchRef::new(
        MeshDrawQueueProfile::new(
            phase,
            MeshDrawGeometrySource::Prepared,
            Mobility::Dynamic,
            false,
            false,
            false,
        ),
        default_pipeline_key(),
        RenderPhaseSortComponents::new(sort_key as f32, sort_key),
        MeshGeometryHandle::test(sort_key),
        MeshDrawArgs::direct_indexed(0, 3),
    )
    .with_gpu_scene_instance_span(sort_key as u32, 1)
    .with_material_textures(MeshBindHandle::test(sort_key + 100))
    .with_material(MeshBindHandle::test(sort_key + 200))
    .with_standard_material(MeshBindHandle::test(sort_key + 300))
}

fn batch_with_depth(phase: MeshDrawQueuePhase, depth: f32, sort_key: u64) -> MeshBatchRef {
    batch(phase, sort_key).with_sort_components(RenderPhaseSortComponents::new(depth, sort_key))
}

fn static_batch(phase: MeshDrawQueuePhase, sort_key: u64) -> MeshBatchRef {
    MeshBatchRef::new(
        MeshDrawQueueProfile::new(
            phase,
            MeshDrawGeometrySource::Prepared,
            Mobility::Static,
            false,
            false,
            false,
        ),
        default_pipeline_key(),
        RenderPhaseSortComponents::new(sort_key as f32, sort_key),
        MeshGeometryHandle::test(sort_key),
        MeshDrawArgs::direct_indexed(0, 3),
    )
    .with_gpu_scene_instance_span(sort_key as u32, 1)
    .with_material_textures(MeshBindHandle::test(sort_key + 100))
    .with_material(MeshBindHandle::test(sort_key + 200))
    .with_standard_material(MeshBindHandle::test(sort_key + 300))
}

fn gpu_driven_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        ..RenderCapabilitySummary::default()
    }
}

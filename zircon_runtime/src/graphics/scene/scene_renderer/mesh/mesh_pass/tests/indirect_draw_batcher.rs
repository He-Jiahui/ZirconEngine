use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
use crate::graphics::scene::resources::default_pipeline_key;

use super::*;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
    MeshPassPipelineKind, MeshPipelineVariantId,
};

#[test]
fn render_gpu_scene_indirect_batcher_groups_by_pipeline_geometry_material() {
    let commands = vec![
        command(10, 1, 2, 3),
        command(20, 3, 3, 2),
        command(30, 6, 1, 1).with_material(MeshBindHandle::test(901)),
    ];

    let batcher = IndirectDrawBatcher::build(&commands, &gpu_driven_capabilities());

    assert_eq!(batcher.fallback_draw_count(), 0);
    assert_eq!(batcher.args_cpu().len(), 3);
    assert_eq!(batcher.batches().len(), 2);
    assert_eq!(batcher.batches()[0].first_command_index, 0);
    assert_eq!(batcher.batches()[0].first_args, 0);
    assert_eq!(batcher.batches()[0].args_count, 2);
    assert_eq!(batcher.batches()[0].draw_count_index, 0);
    assert_eq!(batcher.batches()[0].total_instances, 5);
    assert_eq!(batcher.batches()[1].first_command_index, 2);
    assert_eq!(batcher.batches()[1].first_args, 2);
    assert_eq!(batcher.batches()[1].args_count, 1);
    assert_eq!(batcher.batches()[1].draw_count_index, 1);
    assert_eq!(batcher.batches()[1].total_instances, 1);
    assert_eq!(
        batcher.args_cpu()[1],
        IndexedIndirectArgs {
            index_count: 20,
            instance_count: 2,
            first_index: 3,
            base_vertex: 0,
            first_instance: 3,
        }
    );
    assert_eq!(
        batcher.stats(),
        IndirectDrawBatcherStats {
            batch_count: 2,
            batched_draw_count: 3,
            fallback_draw_count: 0,
            indirect_args_count: 3,
        }
    );
}

#[test]
fn render_gpu_scene_indirect_batcher_keeps_per_draw_indirect_without_multi_draw() {
    let commands = vec![command(10, 1, 2, 1), command(20, 2, 1, 1)];

    let batcher = IndirectDrawBatcher::build(
        &commands,
        &RenderCapabilitySummary {
            supports_indirect_draw: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        },
    );

    assert_eq!(batcher.args_cpu().len(), 2);
    assert_eq!(batcher.batches().len(), 1);
    assert_eq!(batcher.fallback_draw_count(), 0);
    assert_eq!(
        batcher.stats(),
        IndirectDrawBatcherStats {
            batch_count: 1,
            batched_draw_count: 2,
            fallback_draw_count: 0,
            indirect_args_count: 2,
        }
    );
}

#[test]
fn render_gpu_scene_indirect_batcher_uses_direct_draw_when_first_instance_is_unavailable() {
    let commands = vec![command(10, 1, 2, 1)];
    let batcher = IndirectDrawBatcher::build(
        &commands,
        &RenderCapabilitySummary {
            supports_indirect_draw: true,
            ..RenderCapabilitySummary::default()
        },
    );

    assert!(batcher.args_cpu().is_empty());
    assert!(batcher.batches().is_empty());
    assert_eq!(batcher.fallback_draw_count(), 1);
}

#[test]
fn optimization_batch_20260830dv_indirect_args_reserve_command_upper_bound() {
    let source = include_str!("../indirect_draw_batcher.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(production.contains("args_cpu: Vec::with_capacity(commands.len())"));
    assert!(production.contains("batches: Vec<IndirectDrawBatch>"));
    assert!(!production.contains("batches: Vec::with_capacity(commands.len())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dv_indirect_args_capacity_evidence() {
    const FRAME_COUNT: usize = 32_768;
    const COMMAND_COUNT: usize = 256;
    const MARKER: &str = "RUNTIME530_INDIRECT_ARGS_CAPACITY_BENCH_V1";

    let legacy_growth_events = args_growth_events(FRAME_COUNT, COMMAND_COUNT, false);
    let optimized_growth_events = args_growth_events(FRAME_COUNT, COMMAND_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} frames={FRAME_COUNT} commands_per_frame={COMMAND_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn args_growth_events(
    frame_count: usize,
    command_count: usize,
    reserve_upper_bound: bool,
) -> usize {
    let mut growth_events = 0;
    for _ in 0..frame_count {
        let mut args = if reserve_upper_bound {
            Vec::with_capacity(command_count)
        } else {
            Vec::new()
        };
        for command in 0..command_count {
            let previous_capacity = args.capacity();
            args.push(command);
            growth_events += usize::from(args.capacity() != previous_capacity);
        }
    }
    growth_events
}

#[test]
fn render_gpu_scene_indirect_batcher_keeps_existing_indirect_commands_on_fallback_path() {
    let commands = vec![
        command(10, 1, 1, 1),
        MeshDrawCommand::new(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            20,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 4,
                instance_count: 1,
            },
            MeshGeometryHandle::test(7),
            MeshDrawArgs::test_indexed_indirect(99, 32),
        ),
    ];

    let batcher = IndirectDrawBatcher::build(&commands, &gpu_driven_capabilities());

    assert_eq!(batcher.args_cpu().len(), 1);
    assert_eq!(batcher.batches().len(), 1);
    assert_eq!(batcher.fallback_draw_count(), 1);
}

#[test]
fn render_gpu_scene_indirect_batcher_keeps_command_local_gpu_scene_groups_on_direct_path() {
    let commands = vec![
        command(10, 1, 2, 1).with_gpu_scene_bind_group(MeshBindHandle::test(401)),
        command(20, 2, 3, 1),
    ];

    let batcher = IndirectDrawBatcher::build(&commands, &gpu_driven_capabilities());

    assert_eq!(batcher.args_cpu().len(), 1);
    assert_eq!(batcher.batches().len(), 1);
    assert_eq!(batcher.fallback_draw_count(), 1);
    assert_eq!(batcher.batches()[0].first_command_index, 1);
    assert_eq!(batcher.batches()[0].draw_count_index, 0);
}

#[test]
fn render_gpu_scene_indirect_batcher_splits_velocity_draws_by_previous_geometry() {
    let commands = vec![
        velocity_command(10, 1),
        velocity_command(10, 2),
        velocity_command(20, 3),
    ];

    let batcher = IndirectDrawBatcher::build(&commands, &gpu_driven_capabilities());

    assert_eq!(batcher.fallback_draw_count(), 0);
    assert_eq!(batcher.args_cpu().len(), 3);
    assert_eq!(batcher.batches().len(), 2);
    assert_eq!(batcher.batches()[0].args_count, 2);
    assert_eq!(batcher.batches()[1].args_count, 1);
}

fn command(
    index_count: u32,
    first_index: u32,
    first_instance: u32,
    instance_count: u32,
) -> MeshDrawCommand {
    MeshDrawCommand::new(
        RenderPhase::Opaque3d,
        MeshPassPipelineKind::Base,
        default_pipeline_key(),
        MeshPipelineVariantId::new(1),
        u64::from(first_instance),
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index: first_instance,
            instance_count,
        },
        MeshGeometryHandle::test(7),
        MeshDrawArgs::DirectIndexed {
            first_index,
            index_count,
            first_instance,
            instance_count,
        },
    )
    .with_material_textures(MeshBindHandle::test(101))
    .with_material(MeshBindHandle::test(201))
    .with_standard_material(MeshBindHandle::test(301))
}

fn velocity_command(previous_geometry_id: u64, first_instance: u32) -> MeshDrawCommand {
    MeshDrawCommand::new(
        RenderPhase::Opaque3d,
        MeshPassPipelineKind::Velocity,
        default_pipeline_key(),
        MeshPipelineVariantId::new(1),
        u64::from(first_instance),
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index: first_instance,
            instance_count: 1,
        },
        MeshGeometryHandle::test(7),
        MeshDrawArgs::DirectIndexed {
            first_index: 0,
            index_count: 3,
            first_instance,
            instance_count: 1,
        },
    )
    .with_previous_velocity_geometry(MeshGeometryHandle::test(previous_geometry_id))
}

fn gpu_driven_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        ..RenderCapabilitySummary::default()
    }
}

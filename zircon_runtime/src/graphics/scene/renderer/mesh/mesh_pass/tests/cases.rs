use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
use crate::graphics::scene::resources::default_pipeline_key;

use super::*;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshDrawCommandList, MeshGeometryHandle,
    MeshPassPipelineKind, MeshPipelineVariantId,
};

#[test]
fn phase_plan_seals_batch_stats_with_the_execution_artifact() {
    let commands = vec![command(0), command(1)];

    let (plan, stats) = MeshIndirectDrawPlan::build(&commands, &gpu_driven_capabilities());

    let plan = plan.expect("GPU-driven commands produce an indirect plan");
    assert_eq!(plan.batcher.args_cpu().len(), 2);
    assert_eq!(plan.compaction_plan.metadata_count(), 2);
    assert_eq!(stats.batch_count, 1);
    assert_eq!(stats.batched_draw_count, 2);
    assert_eq!(stats.fallback_draw_count, 0);
    assert_eq!(stats.indirect_args_count, 2);
}

#[test]
fn disabled_gpu_driven_plan_reports_fallback_without_an_execution_artifact() {
    let commands = vec![command(0), command(1)];

    let (plan, stats) = MeshIndirectDrawPlan::build(&commands, &RenderCapabilitySummary::default());

    assert!(plan.is_none());
    assert_eq!(stats.batch_count, 0);
    assert_eq!(stats.batched_draw_count, 0);
    assert_eq!(stats.fallback_draw_count, 2);
    assert_eq!(stats.indirect_args_count, 0);
}

#[test]
fn execution_parts_move_the_sealed_batches_without_rebuilding_them() {
    let commands = vec![command(0), command(1)];
    let (plan, _) = MeshIndirectDrawPlan::build(&commands, &gpu_driven_capabilities());
    let plan = plan.expect("indirect plan");

    let (args, batches) = plan.batcher.into_execution_parts();

    assert_eq!(args.len(), 2);
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].args_count, 2);
}

#[test]
fn pass_plan_aggregates_sealed_stats_across_render_phases() {
    let commands = MeshDrawCommandList::from_commands(vec![
        command_in_phase(RenderPhase::Opaque3d, 0),
        command_in_phase(RenderPhase::Transparent3d, 1),
    ]);
    let buffers = MeshPassCommandBuffers::from_cached_command_hits(commands, Default::default());

    let plans = MeshPassIndirectDrawPlans::build(&buffers, &gpu_driven_capabilities());

    assert_eq!(plans.stats().batched_draw_count, 2);
    assert_eq!(plans.stats().batch_count, 2);
    assert_eq!(plans.stats().fallback_draw_count, 0);
}

fn command(first_instance: u32) -> MeshDrawCommand {
    command_in_phase(RenderPhase::Opaque3d, first_instance)
}

fn command_in_phase(phase: RenderPhase, first_instance: u32) -> MeshDrawCommand {
    MeshDrawCommand::new(
        phase,
        MeshPassPipelineKind::Base,
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
}

fn gpu_driven_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        ..RenderCapabilitySummary::default()
    }
}

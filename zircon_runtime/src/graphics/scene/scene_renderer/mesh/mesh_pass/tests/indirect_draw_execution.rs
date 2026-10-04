use super::{MeshIndirectArgsSnapshot, INDEXED_INDIRECT_ARGS_STRIDE_BYTES};
use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshDrawCommandList, MeshGeometryHandle,
    MeshIndirectDrawWorkspace, MeshPassCommandBuffers, MeshPassIndirectDrawPlans,
    MeshPassPipelineKind, MeshPipelineVariantId,
};
use zr_rhi_wgpu::WgpuBufferUploadBatch;

#[test]
fn mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer() {
    let source = include_str!("../indirect_draw_execution.rs");

    assert_eq!(INDEXED_INDIRECT_ARGS_STRIDE_BYTES, 20);
    assert!(source.contains("from_prepared_plan"));
    assert!(source.contains("args_buffer: Arc<wgpu::Buffer>"));
    // BUG: [CR-R02-runtime_wave12_graphics_mesh_submission-0002] 此测试完整包含自身源码，负断言 needle 就在本行，前置断言成立后必失败；应只检查生产部分。
    assert!(!source.contains("create_buffer_init"));
}

#[test]
fn mesh_indirect_args_snapshot_counts_zeroed_instance_args() {
    let snapshot = MeshIndirectArgsSnapshot::from_args_and_draw_counts(
        vec![
            indirect_args(10, 4, 2),
            indirect_args(20, 0, 8),
            indirect_args(30, 6, 12),
            indirect_args(40, 0, 18),
        ],
        vec![1, 2],
    );

    assert_eq!(snapshot.args_count(), 4);
    assert_eq!(snapshot.compacted_draw_count(), 3);
    assert_eq!(snapshot.zero_instance_arg_count(), 2);
    assert_eq!(snapshot.remaining_instance_count(), 10);
}

#[test]
fn mesh_indirect_draw_execution_routes_readback_through_the_product_diagnostic_batch() {
    let source = include_str!("../indirect_draw_execution.rs");

    assert!(source.contains("backend.enqueue_product_diagnostic_buffer"));
    assert!(source.contains("self.replay_args_buffer()"));
    assert!(source.contains("self.compaction_resources.draw_count_buffer()"));
    assert!(source.contains("SharedReadbackBytes"));
    assert!(source.contains("decode_indexed_indirect_args"));
    // BUG: [CR-R02-runtime_wave12_graphics_mesh_submission-0002] 此测试也完整包含自身源码，负断言 needle 已由本行提供，前置断言均成立后必失败；后续 map_async 断言尚未到达。
    assert!(!source.contains("request_readback_external"));
    assert!(!source.contains("map_async"));
}

#[test]
fn mesh_indirect_draw_execution_builds_compaction_plan_from_uploaded_args() {
    let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
        return;
    };
    let mut commands = MeshDrawCommandList::new();
    commands.push(command(10, 1, 2, 3));
    commands.push(command(20, 4, 8, 2));
    let command_buffers =
        MeshPassCommandBuffers::from_cached_command_hits(commands, Default::default());
    let capabilities = gpu_driven_capabilities();
    let plans = MeshPassIndirectDrawPlans::build(&command_buffers, &capabilities);
    let mut workspace = MeshIndirectDrawWorkspace::default();
    let (executions, _, mut prepared_upload) =
        workspace.prepare(&backend.device, &capabilities, plans);
    let execution = executions.opaque().expect("opaque indirect execution");
    let mut uploads = WgpuBufferUploadBatch::new();
    prepared_upload.append_to(&mut uploads);
    backend
        .enqueue_copy_buffer_upload_batch(uploads)
        .expect("indirect args and compaction metadata upload should be accepted");
    let encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-test-indirect-compaction-upload-flush"),
        });
    backend
        .submit_graphics_command_buffers(vec![encoder.finish()])
        .expect("indirect uploads should submit before their shadows commit");
    assert_eq!(prepared_upload.commit(&mut workspace), 1);

    let plan = execution.compaction_plan();
    assert_eq!(plan.metadata_count(), 2);
    assert_eq!(plan.visible_instance_capacity(), 5);
    assert_eq!(plan.metadata()[0].visible_instance_base, 0);
    assert_eq!(plan.metadata()[0].source_first_instance, 2);
    assert_eq!(plan.metadata()[0].source_instance_count, 3);
    assert_eq!(plan.metadata()[1].visible_instance_base, 3);
    assert_eq!(plan.metadata()[1].source_first_instance, 8);
    assert_eq!(plan.metadata()[1].source_instance_count, 2);

    let resources = execution.compaction_resources();
    assert_eq!(
        resources.metadata_buffer_byte_size(),
        plan.metadata_buffer_byte_size()
    );
    assert_eq!(
        resources.visible_instance_index_buffer_byte_size(),
        plan.visible_instance_index_buffer_byte_size()
    );
    assert_eq!(resources.visible_instance_index_capacity(), 5);
    assert_eq!(
        resources.visible_instance_index_buffer_allocation_byte_size(),
        20
    );
    assert_eq!(resources.compacted_indirect_args_buffer_byte_size(), 40);
    assert_eq!(resources.draw_count_buffer_byte_size(), 4);
    assert_eq!(resources.draw_count_capacity(), 1);
}

fn indirect_args(
    index_count: u32,
    instance_count: u32,
    first_instance: u32,
) -> IndexedIndirectArgs {
    IndexedIndirectArgs {
        index_count,
        instance_count,
        first_index: 0,
        base_vertex: 0,
        first_instance,
    }
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
}

fn gpu_driven_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        ..RenderCapabilitySummary::default()
    }
}

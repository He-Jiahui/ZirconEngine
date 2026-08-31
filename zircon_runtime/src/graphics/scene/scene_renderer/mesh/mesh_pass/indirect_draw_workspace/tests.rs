use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
use crate::graphics::scene::resources::default_pipeline_key;

use super::*;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle, MeshPassPipelineKind,
    MeshPipelineVariantId,
};
use zr_rhi_wgpu::WgpuBufferUploadBatch;

#[test]
fn stable_phase_workspace_reuses_buffers_and_skips_unchanged_uploads() {
    let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
        return;
    };
    let capabilities = gpu_driven_capabilities();
    let commands = [command(0), command(1)];
    let mut workspace = MeshIndirectPhaseWorkspace::default();

    let (first_plan, _) = MeshIndirectDrawPlan::build(&commands, &capabilities);
    let mut first_uploads = WgpuBufferUploadBatch::new();
    let (first_execution, first_stats, first_commit) = workspace.prepare(
        MeshIndirectPhase::Opaque,
        &backend.device,
        "zircon-test-indirect-workspace",
        first_plan.expect("first plan"),
        &capabilities,
        &mut first_uploads,
    );
    assert!(workspace.commit_prepared_upload(first_commit.expect("first commit")));
    let (second_plan, _) = MeshIndirectDrawPlan::build(&commands, &capabilities);
    let mut second_uploads = WgpuBufferUploadBatch::new();
    let (second_execution, second_stats, second_commit) = workspace.prepare(
        MeshIndirectPhase::Opaque,
        &backend.device,
        "zircon-test-indirect-workspace",
        second_plan.expect("second plan"),
        &capabilities,
        &mut second_uploads,
    );

    assert_eq!(first_stats.created_buffer_count, 5);
    assert!(first_stats.uploaded_byte_count > 0);
    assert_eq!(first_stats.upload_range_count, 2);
    assert_eq!(second_stats.created_buffer_count, 0);
    assert_eq!(second_stats.uploaded_byte_count, 0);
    assert_eq!(second_stats.upload_range_count, 0);
    assert!(second_commit.is_none());
    assert_eq!(
        first_execution.resource_identity(),
        second_execution.resource_identity()
    );
}

#[test]
fn changed_phase_workspace_uploads_only_the_dirty_args_range() {
    let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
        return;
    };
    let capabilities = gpu_driven_capabilities();
    let mut workspace = MeshIndirectPhaseWorkspace::default();
    let initial_commands = [
        command_with_index_count(0, 3),
        command_with_index_count(1, 3),
    ];
    let (initial_plan, _) = MeshIndirectDrawPlan::build(&initial_commands, &capabilities);
    let mut initial_uploads = WgpuBufferUploadBatch::new();
    let (_, _, initial_commit) = workspace.prepare(
        MeshIndirectPhase::Opaque,
        &backend.device,
        "zircon-test-indirect-dirty-range",
        initial_plan.expect("initial plan"),
        &capabilities,
        &mut initial_uploads,
    );
    assert!(workspace.commit_prepared_upload(initial_commit.expect("initial commit")));

    let changed_commands = [
        command_with_index_count(0, 3),
        command_with_index_count(1, 6),
    ];
    let (changed_plan, _) = MeshIndirectDrawPlan::build(&changed_commands, &capabilities);
    let mut changed_uploads = WgpuBufferUploadBatch::new();
    let (_, stats, _) = workspace.prepare(
        MeshIndirectPhase::Opaque,
        &backend.device,
        "zircon-test-indirect-dirty-range",
        changed_plan.expect("changed plan"),
        &capabilities,
        &mut changed_uploads,
    );

    assert_eq!(stats.created_buffer_count, 0);
    assert_eq!(
        stats.uploaded_byte_count,
        INDEXED_INDIRECT_ARGS_STRIDE_BYTES
    );
    assert_eq!(stats.upload_range_count, 1);
}

#[test]
fn growing_phase_workspace_advances_resource_revision_without_changing_workspace_id() {
    let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
        return;
    };
    let capabilities = gpu_driven_capabilities();
    let mut workspace = MeshIndirectPhaseWorkspace::default();
    let (small_plan, _) = MeshIndirectDrawPlan::build(&[command(0)], &capabilities);
    let mut small_uploads = WgpuBufferUploadBatch::new();
    let (small, _, _) = workspace.prepare(
        MeshIndirectPhase::Opaque,
        &backend.device,
        "zircon-test-indirect-resource-revision",
        small_plan.expect("small plan"),
        &capabilities,
        &mut small_uploads,
    );
    let large_commands = (0..4).map(command).collect::<Vec<_>>();
    let (large_plan, _) = MeshIndirectDrawPlan::build(&large_commands, &capabilities);
    let mut large_uploads = WgpuBufferUploadBatch::new();
    let (large, _, _) = workspace.prepare(
        MeshIndirectPhase::Opaque,
        &backend.device,
        "zircon-test-indirect-resource-revision",
        large_plan.expect("large plan"),
        &capabilities,
        &mut large_uploads,
    );

    assert_eq!(
        small.resource_identity().workspace_id(),
        large.resource_identity().workspace_id()
    );
    assert!(
        large.resource_identity().resource_revision()
            > small.resource_identity().resource_revision()
    );
}

#[test]
fn uncommitted_new_buffer_retries_a_full_upload() {
    let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
        return;
    };
    let capabilities = gpu_driven_capabilities();
    let commands = [command(0), command(1)];
    let mut workspace = MeshIndirectPhaseWorkspace::default();

    let (first_plan, _) = MeshIndirectDrawPlan::build(&commands, &capabilities);
    let mut first_uploads = WgpuBufferUploadBatch::new();
    let (_, first_stats, _) = workspace.prepare(
        MeshIndirectPhase::Opaque,
        &backend.device,
        "zircon-test-indirect-retry",
        first_plan.expect("first plan"),
        &capabilities,
        &mut first_uploads,
    );

    let (retry_plan, _) = MeshIndirectDrawPlan::build(&commands, &capabilities);
    let mut retry_uploads = WgpuBufferUploadBatch::new();
    let (_, retry_stats, retry_commit) = workspace.prepare(
        MeshIndirectPhase::Opaque,
        &backend.device,
        "zircon-test-indirect-retry",
        retry_plan.expect("retry plan"),
        &capabilities,
        &mut retry_uploads,
    );

    assert_eq!(first_stats.upload_range_count, 2);
    assert_eq!(retry_stats.created_buffer_count, 0);
    assert_eq!(
        retry_stats.uploaded_byte_count,
        first_stats.uploaded_byte_count
    );
    assert_eq!(
        retry_stats.upload_range_count,
        first_stats.upload_range_count
    );
    assert!(retry_commit.is_some());
}

#[test]
fn mesh_indirect_prepare_has_no_queue_or_direct_write_authority() {
    let workspace_source = include_str!("../indirect_draw_workspace.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("workspace production source");
    let range_upload_source = include_str!("../indirect_buffer_upload.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("range upload production source");
    let compaction_source = include_str!("../indirect_compaction_resources.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("compaction production source");

    for source in [workspace_source, range_upload_source, compaction_source] {
        assert!(!source.contains("wgpu::Queue"));
        assert!(!source.contains("queue.write_buffer"));
    }
    assert!(workspace_source.contains("MeshIndirectWorkspacePreparedUpload"));
    assert!(workspace_source.contains("committed frame upload batch"));
}

fn command(first_instance: u32) -> MeshDrawCommand {
    command_with_index_count(first_instance, 3)
}

fn command_with_index_count(first_instance: u32, index_count: u32) -> MeshDrawCommand {
    MeshDrawCommand::new(
        RenderPhase::Opaque3d,
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
            index_count,
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

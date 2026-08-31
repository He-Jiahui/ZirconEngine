use std::sync::Arc;

use crate::virtual_geometry::types::VirtualGeometryClusterSelection;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryExecutionDraw,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
};

use super::super::virtual_geometry_executed_cluster_selection_pass::execute_virtual_geometry_executed_cluster_selection_pass;
use super::super::virtual_geometry_hardware_rasterization_pass::execute_virtual_geometry_hardware_rasterization_pass;
use super::super::virtual_geometry_node_and_cluster_cull_pass::execute_virtual_geometry_node_and_cluster_cull_pass;
use super::super::virtual_geometry_visbuffer64_pass::execute_virtual_geometry_visbuffer64_pass;
use super::execution_owned_buffers::{
    build_execution_authority_buffer, build_execution_submission_buffer,
};
use super::execution_segments::{collect_execution_segments, execution_segment_summary};
use super::virtual_geometry_indirect_stats::VirtualGeometryIndirectStats;
use crate::virtual_geometry::renderer::{VirtualGeometryGpuResources, VirtualGeometryRenderFrame};

struct ExecutionDrawProjection<'a> {
    indirect_execution_draws: Vec<&'a RenderVirtualGeometryExecutionDraw>,
    has_execution_args_buffer: bool,
    draw_submission_order: Vec<(Option<u32>, u64, u32)>,
    execution_indirect_offsets: Vec<u64>,
    draw_submission_records: Vec<(u64, u32, u32, usize)>,
    draw_submission_token_records: Vec<(u64, u32, u32, u32, usize)>,
}

fn collect_execution_draw_projection(
    execution_draws: &[RenderVirtualGeometryExecutionDraw],
) -> ExecutionDrawProjection<'_> {
    let capacity = execution_draws.len();
    let mut projection = ExecutionDrawProjection {
        indirect_execution_draws: Vec::with_capacity(capacity),
        has_execution_args_buffer: false,
        draw_submission_order: Vec::with_capacity(capacity),
        execution_indirect_offsets: Vec::with_capacity(capacity),
        draw_submission_records: Vec::with_capacity(capacity),
        draw_submission_token_records: Vec::with_capacity(capacity),
    };

    for draw in execution_draws {
        if let Some(record) = draw.submission_order_record {
            projection.draw_submission_order.push(record);
        }
        if let Some(record) = draw.draw_submission_record {
            projection.draw_submission_records.push(record);
        }
        if let Some(record) = draw.draw_submission_token_record {
            projection.draw_submission_token_records.push(record);
        }
        if draw.uses_indirect_draw {
            projection.has_execution_args_buffer |= draw.indirect_args_buffer_available;
            projection.indirect_execution_draws.push(draw);
            projection
                .execution_indirect_offsets
                .push(draw.indirect_args_offset);
        }
    }

    projection
}

#[allow(clippy::too_many_arguments)]
pub(in crate::virtual_geometry::renderer::root_render_passes) fn collect_virtual_geometry_indirect_stats(
    virtual_geometry_resources: &VirtualGeometryGpuResources,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    visbuffer64_pass_enabled: bool,
    frame: &VirtualGeometryRenderFrame,
    cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
    previous_node_and_cluster_cull_global_state: Option<
        &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    >,
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    execution_draws: &[RenderVirtualGeometryExecutionDraw],
    args_buffer: Option<Arc<wgpu::Buffer>>,
    args_count: u32,
    segment_count: u32,
    submission_buffer: Option<Arc<wgpu::Buffer>>,
    authority_buffer: Option<Arc<wgpu::Buffer>>,
    draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    segment_buffer: Option<Arc<wgpu::Buffer>>,
) -> VirtualGeometryIndirectStats {
    virtual_geometry_indirect_stats(
        virtual_geometry_resources,
        device,
        encoder,
        visbuffer64_pass_enabled,
        frame,
        cull_input,
        previous_node_and_cluster_cull_global_state,
        cluster_selections,
        execution_draws,
        args_buffer,
        args_count,
        segment_count,
        submission_buffer,
        authority_buffer,
        draw_ref_buffer,
        segment_buffer,
    )
}

#[allow(clippy::too_many_arguments)]
fn virtual_geometry_indirect_stats(
    virtual_geometry_resources: &VirtualGeometryGpuResources,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    visbuffer64_pass_enabled: bool,
    frame: &VirtualGeometryRenderFrame,
    cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
    previous_node_and_cluster_cull_global_state: Option<
        &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    >,
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    execution_draws: &[RenderVirtualGeometryExecutionDraw],
    args_buffer: Option<Arc<wgpu::Buffer>>,
    args_count: u32,
    segment_count: u32,
    submission_buffer: Option<Arc<wgpu::Buffer>>,
    authority_buffer: Option<Arc<wgpu::Buffer>>,
    draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    segment_buffer: Option<Arc<wgpu::Buffer>>,
) -> VirtualGeometryIndirectStats {
    let ExecutionDrawProjection {
        indirect_execution_draws,
        has_execution_args_buffer,
        draw_submission_order,
        execution_indirect_offsets,
        draw_submission_records,
        draw_submission_token_records,
    } = collect_execution_draw_projection(execution_draws);
    let draw_count = indirect_execution_draws.len() as u32;
    let execution_segments = collect_execution_segments(&indirect_execution_draws);
    let execution_summary = execution_segment_summary(&execution_segments, draw_count);
    let buffer_count = u32::from(has_execution_args_buffer && args_buffer.is_some());
    let execution_args_buffer = has_execution_args_buffer
        .then(|| args_buffer.as_ref().map(Arc::clone))
        .flatten();
    let node_and_cluster_cull_pass = execute_virtual_geometry_node_and_cluster_cull_pass(
        device,
        encoder,
        virtual_geometry_resources,
        visbuffer64_pass_enabled,
        frame,
        cull_input,
        previous_node_and_cluster_cull_global_state,
    );
    let executed_cluster_selection_pass = execute_virtual_geometry_executed_cluster_selection_pass(
        device,
        visbuffer64_pass_enabled,
        cluster_selections,
        &indirect_execution_draws,
        frame.extract.geometry.virtual_geometry.as_ref(),
        &node_and_cluster_cull_pass,
    );
    let hardware_rasterization_pass = execute_virtual_geometry_hardware_rasterization_pass(
        device,
        visbuffer64_pass_enabled,
        &executed_cluster_selection_pass,
    );
    let visbuffer64_pass = execute_virtual_geometry_visbuffer64_pass(
        device,
        visbuffer64_pass_enabled,
        &executed_cluster_selection_pass,
    );
    let (
        executed_selected_clusters,
        executed_selected_cluster_source,
        executed_selected_cluster_count,
        executed_selected_cluster_buffer,
    ) = executed_cluster_selection_pass.into_indirect_stats_parts();
    let execution_submission_buffer = build_execution_submission_buffer(
        device,
        encoder,
        &indirect_execution_draws,
        submission_buffer.as_ref(),
    );
    let execution_authority_buffer = build_execution_authority_buffer(
        device,
        encoder,
        &indirect_execution_draws,
        authority_buffer.as_ref(),
    );
    VirtualGeometryIndirectStats::new(
        draw_count,
        buffer_count,
        segment_count,
        execution_summary.segment_count(),
        execution_summary.page_count(),
        execution_summary.resident_segment_count(),
        execution_summary.pending_segment_count(),
        execution_summary.missing_segment_count(),
        execution_summary.repeated_draw_count(),
        execution_indirect_offsets,
        execution_segments,
        executed_selected_clusters,
        executed_selected_cluster_source,
        executed_selected_cluster_count,
        executed_selected_cluster_buffer,
        node_and_cluster_cull_pass,
        hardware_rasterization_pass,
        visbuffer64_pass,
        draw_submission_order,
        draw_submission_records,
        draw_submission_token_records,
        args_buffer,
        args_count,
        submission_buffer,
        authority_buffer,
        draw_ref_buffer,
        segment_buffer,
        execution_submission_buffer,
        execution_args_buffer,
        execution_authority_buffer,
    )
}

#[cfg(test)]
mod draw_projection_tests;

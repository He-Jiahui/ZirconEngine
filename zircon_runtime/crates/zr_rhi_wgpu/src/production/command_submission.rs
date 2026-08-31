use zr_rhi::{CommandList, CommandListCommand, DiagnosticPassQueryScope, RhiError};

use super::command_encoder::{
    encode_buffer_copy, encode_buffer_to_texture_copy, encode_compute_dispatch,
    encode_compute_dispatch_indirect, encode_compute_pass, encode_render_pass,
    encode_texture_to_buffer_copy, encode_texture_to_texture_copy, pop_debug_group,
    validate_bind_group_slot, validate_debug_label, validate_index_buffer, validate_vertex_buffer,
    BufferBinding, DebugGroupScope, EncoderState, IndexBufferBinding,
};
use super::diagnostics::WgpuDiagnosticQueryFrame;
use super::WgpuResourceRegistry;
use crate::bind_group_validation::validate_bind_group_dynamic_offsets;

/// Traverses one neutral command list and delegates native encode state to the
/// specialized command and render-pass owners.
pub(crate) fn encode_command_list(
    device: &wgpu::Device,
    registry: &WgpuResourceRegistry,
    command_list: &dyn CommandList,
    diagnostic_frame: Option<&WgpuDiagnosticQueryFrame>,
    limits: &zr_rhi::RenderDeviceLimits,
) -> Result<wgpu::CommandBuffer, RhiError> {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: command_list.label(),
    });
    let commands = command_list.recorded_commands();
    let mut state = EncoderState::default();
    let mut debug_groups = Vec::new();
    let mut command_index = 0;

    while command_index < commands.len() {
        match &commands[command_index] {
            CommandListCommand::BeginRenderPass {
                label,
                color_attachments,
                depth_stencil_attachment,
            } => {
                super::command_encoder::require_graphics_queue(
                    command_list.queue_class(),
                    "begin_render_pass",
                )?;
                command_index = encode_render_pass(
                    &mut encoder,
                    registry,
                    commands,
                    command_index,
                    label,
                    color_attachments,
                    *depth_stencil_attachment,
                    &mut state,
                    &mut debug_groups,
                    command_list.queue_class(),
                    None,
                    limits,
                )?;
            }
            CommandListCommand::BeginRenderPassWithDiagnostics {
                label,
                color_attachments,
                depth_stencil_attachment,
                diagnostic_scope,
            } => {
                write_timestamp_begin(&mut encoder, diagnostic_frame, *diagnostic_scope)?;
                command_index = encode_render_pass(
                    &mut encoder,
                    registry,
                    commands,
                    command_index,
                    label,
                    color_attachments,
                    *depth_stencil_attachment,
                    &mut state,
                    &mut debug_groups,
                    command_list.queue_class(),
                    diagnostic_frame.zip(Some(*diagnostic_scope)),
                    limits,
                )?;
                write_timestamp_end(&mut encoder, diagnostic_frame, *diagnostic_scope)?;
            }
            CommandListCommand::EndRenderPass => {
                return Err(RhiError::InvalidRenderPass {
                    reason: "end_render_pass requires an active render pass".to_string(),
                });
            }
            CommandListCommand::BeginComputePass { label } => {
                command_index = encode_compute_pass(
                    &mut encoder,
                    registry,
                    commands,
                    command_index,
                    label,
                    &mut state,
                    &mut debug_groups,
                    command_list.queue_class(),
                    None,
                    limits,
                )?;
            }
            CommandListCommand::BeginComputePassWithDiagnostics {
                label,
                diagnostic_scope,
            } => {
                write_timestamp_begin(&mut encoder, diagnostic_frame, *diagnostic_scope)?;
                command_index = encode_compute_pass(
                    &mut encoder,
                    registry,
                    commands,
                    command_index,
                    label,
                    &mut state,
                    &mut debug_groups,
                    command_list.queue_class(),
                    diagnostic_frame.zip(Some(*diagnostic_scope)),
                    limits,
                )?;
                write_timestamp_end(&mut encoder, diagnostic_frame, *diagnostic_scope)?;
            }
            CommandListCommand::EndComputePass => {
                return Err(RhiError::InvalidComputePass {
                    reason: "end_compute_pass requires an active compute pass".to_string(),
                });
            }
            CommandListCommand::DebugMarker { label } => {
                validate_debug_label(label, "debug marker")?;
                encoder.insert_debug_marker(label);
                command_index += 1;
            }
            CommandListCommand::PushDebugGroup { label } => {
                validate_debug_label(label, "debug group")?;
                encoder.push_debug_group(label);
                debug_groups.push(DebugGroupScope::CommandEncoder);
                command_index += 1;
            }
            CommandListCommand::PopDebugGroup => {
                pop_debug_group(&mut debug_groups, DebugGroupScope::CommandEncoder)?;
                encoder.pop_debug_group();
                command_index += 1;
            }
            CommandListCommand::CopyBufferToBuffer {
                source,
                destination,
                source_offset,
                destination_offset,
                size,
            } => {
                encode_buffer_copy(
                    &mut encoder,
                    registry,
                    *source,
                    *destination,
                    *source_offset,
                    *destination_offset,
                    *size,
                )?;
                command_index += 1;
            }
            CommandListCommand::CopyBufferToTexture {
                source,
                destination,
                source_offset,
                bytes_per_row,
                region,
            } => {
                encode_buffer_to_texture_copy(
                    &mut encoder,
                    registry,
                    *source,
                    *destination,
                    *source_offset,
                    *bytes_per_row,
                    *region,
                )?;
                command_index += 1;
            }
            CommandListCommand::CopyTextureToBuffer {
                source,
                destination,
                destination_offset,
                bytes_per_row,
                region,
            } => {
                encode_texture_to_buffer_copy(
                    &mut encoder,
                    registry,
                    *source,
                    *destination,
                    *destination_offset,
                    *bytes_per_row,
                    *region,
                )?;
                command_index += 1;
            }
            CommandListCommand::CopyTextureToTexture {
                source,
                destination,
                source_region,
                destination_region,
            } => {
                encode_texture_to_texture_copy(
                    &mut encoder,
                    registry,
                    *source,
                    *destination,
                    *source_region,
                    *destination_region,
                )?;
                command_index += 1;
            }
            CommandListCommand::SetPipeline { pipeline } => {
                state.set_pipeline(*pipeline, registry.pipeline_desc(*pipeline)?);
                command_index += 1;
            }
            CommandListCommand::SetBindGroup {
                slot,
                bind_group,
                dynamic_offsets,
            } => {
                let bind_group_desc = registry.bind_group_desc(*bind_group)?;
                validate_bind_group_dynamic_offsets(
                    registry,
                    *bind_group,
                    &bind_group_desc,
                    dynamic_offsets,
                    limits,
                )?;
                if let Some((_, pipeline)) = state.current_pipeline.as_ref() {
                    validate_bind_group_slot(registry, pipeline, *slot, *bind_group)?;
                }
                state.set_bind_group(*slot, *bind_group, dynamic_offsets.clone());
                command_index += 1;
            }
            CommandListCommand::SetViewport { .. } | CommandListCommand::SetScissorRect { .. } => {
                return Err(RhiError::InvalidRenderPass {
                    reason: "viewport and scissor commands require an active render pass"
                        .to_string(),
                });
            }
            CommandListCommand::SetVertexBuffer {
                slot,
                buffer,
                offset,
                size,
            } => {
                validate_vertex_buffer(registry, *buffer, *offset, *size)?;
                state.vertex_buffers.insert(
                    *slot,
                    BufferBinding {
                        handle: *buffer,
                        offset: *offset,
                        size: *size,
                    },
                );
                command_index += 1;
            }
            CommandListCommand::SetIndexBuffer {
                buffer,
                offset,
                size,
                format,
            } => {
                validate_index_buffer(registry, *buffer, *offset, *size, *format)?;
                state.index_buffer = Some(IndexBufferBinding {
                    handle: *buffer,
                    offset: *offset,
                    size: *size,
                    format: *format,
                });
                command_index += 1;
            }
            CommandListCommand::Draw { .. }
            | CommandListCommand::DrawIndexed { .. }
            | CommandListCommand::DrawIndirect { .. }
            | CommandListCommand::DrawIndexedIndirect { .. }
            | CommandListCommand::MultiDrawIndirect { .. }
            | CommandListCommand::MultiDrawIndirectCount { .. }
            | CommandListCommand::MultiDrawIndexedIndirect { .. }
            | CommandListCommand::MultiDrawIndexedIndirectCount { .. } => {
                return Err(RhiError::InvalidRenderPass {
                    reason: "draw commands require an active render pass".to_string(),
                });
            }
            CommandListCommand::DispatchCompute { x, y, z } => {
                encode_compute_dispatch(
                    &mut encoder,
                    registry,
                    &state,
                    command_list.queue_class(),
                    *x,
                    *y,
                    *z,
                )?;
                command_index += 1;
            }
            CommandListCommand::DispatchComputeIndirect { arguments, offset } => {
                encode_compute_dispatch_indirect(
                    &mut encoder,
                    registry,
                    &state,
                    command_list.queue_class(),
                    *arguments,
                    *offset,
                )?;
                command_index += 1;
            }
        }
    }

    if !debug_groups.is_empty() {
        return Err(RhiError::InvalidDebugMarker {
            reason: "command list ended with an active debug group".to_string(),
        });
    }

    Ok(encoder.finish())
}

fn write_timestamp_begin(
    encoder: &mut wgpu::CommandEncoder,
    diagnostic_frame: Option<&WgpuDiagnosticQueryFrame>,
    scope: DiagnosticPassQueryScope,
) -> Result<(), RhiError> {
    let Some(diagnostic_frame) = diagnostic_frame else {
        return Ok(());
    };
    let Some(timestamp) = scope.timestamp() else {
        return Ok(());
    };
    let query_set = diagnostic_frame
        .timestamp_query_set()
        .ok_or(RhiError::DiagnosticQueryPlanRequired)?;
    encoder.write_timestamp(query_set, timestamp.begin_query());
    Ok(())
}

fn write_timestamp_end(
    encoder: &mut wgpu::CommandEncoder,
    diagnostic_frame: Option<&WgpuDiagnosticQueryFrame>,
    scope: DiagnosticPassQueryScope,
) -> Result<(), RhiError> {
    let Some(diagnostic_frame) = diagnostic_frame else {
        return Ok(());
    };
    let Some(timestamp) = scope.timestamp() else {
        return Ok(());
    };
    let query_set = diagnostic_frame
        .timestamp_query_set()
        .ok_or(RhiError::DiagnosticQueryPlanRequired)?;
    encoder.write_timestamp(query_set, timestamp.end_query());
    Ok(())
}

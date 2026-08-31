use zr_rhi::{BufferUsage, CommandListCommand, RenderQueueClass, RhiError};

mod copy_commands;
mod render_state;

use self::render_state::{
    CommandResourceLookup, IndexBufferBinding, RecordedRenderState, ensure_binding_range,
    validate_bind_group_slot, validate_index_range,
};
use super::bind_group_validation::{validate_bind_group_desc, validate_bind_group_dynamic_offsets};
use super::device::DeterministicRhiContractDeviceState;
use super::indirect_validation::{
    IndirectArgumentKind, validate_indirect_arguments, validate_indirect_count_buffer,
};
use super::render_pass_validation::{ActiveRenderPass, validate_render_pass_attachments};
use super::resource_validation::ensure_buffer_usage;

pub(super) fn validate_recorded_commands(
    state: &DeterministicRhiContractDeviceState,
    commands: &[CommandListCommand],
    queue_class: RenderQueueClass,
    limits: &zr_rhi::RenderDeviceLimits,
) -> Result<(), RhiError> {
    let mut render_state = RecordedRenderState::default();
    let mut active_render_pass: Option<ActiveRenderPass> = None;
    let mut active_compute_pass = false;
    let mut debug_group_stack = Vec::new();
    for command in commands {
        if copy_commands::validate(state, command, &active_render_pass, active_compute_pass)? {
            continue;
        }
        match command {
            CommandListCommand::DebugMarker { label } => {
                validate_debug_label(label, "debug marker")?;
            }
            CommandListCommand::PushDebugGroup { label } => {
                validate_debug_label(label, "debug group")?;
                debug_group_stack.push(DebugGroupScope::from_active_scopes(
                    active_render_pass.is_some(),
                    active_compute_pass,
                ));
            }
            CommandListCommand::PopDebugGroup => {
                let expected_scope = DebugGroupScope::from_active_scopes(
                    active_render_pass.is_some(),
                    active_compute_pass,
                );
                match debug_group_stack.last().copied() {
                    Some(scope) if scope == expected_scope => {
                        debug_group_stack.pop();
                    }
                    Some(DebugGroupScope::CommandEncoder) => {
                        return Err(RhiError::InvalidDebugMarker {
                            reason: match expected_scope {
                                DebugGroupScope::RenderPass => "pop_debug_group must close a debug group recorded outside the active render pass".to_string(),
                                DebugGroupScope::ComputePass => "pop_debug_group must close a debug group recorded outside the active compute pass".to_string(),
                                DebugGroupScope::CommandEncoder => unreachable!(),
                            },
                        });
                    }
                    Some(DebugGroupScope::RenderPass) => {
                        return Err(RhiError::InvalidDebugMarker {
                            reason: match expected_scope {
                                DebugGroupScope::CommandEncoder => "pop_debug_group must close a debug group recorded in the active render pass".to_string(),
                                DebugGroupScope::ComputePass => "pop_debug_group must close a debug group recorded in the active render pass".to_string(),
                                DebugGroupScope::RenderPass => unreachable!(),
                            },
                        });
                    }
                    Some(DebugGroupScope::ComputePass) => {
                        return Err(RhiError::InvalidDebugMarker {
                            reason: match expected_scope {
                                DebugGroupScope::CommandEncoder => "pop_debug_group must close a debug group recorded in the active compute pass".to_string(),
                                DebugGroupScope::RenderPass => "pop_debug_group must close a debug group recorded in the active compute pass".to_string(),
                                DebugGroupScope::ComputePass => unreachable!(),
                            },
                        });
                    }
                    None => {
                        return Err(RhiError::InvalidDebugMarker {
                            reason: "pop_debug_group requires an active debug group".to_string(),
                        });
                    }
                }
            }
            CommandListCommand::CopyBufferToBuffer { .. }
            | CommandListCommand::CopyBufferToTexture { .. }
            | CommandListCommand::CopyTextureToBuffer { .. }
            | CommandListCommand::CopyTextureToTexture { .. } => {
                unreachable!("copy commands are handled by command_validation::copy_commands")
            }
            CommandListCommand::BeginRenderPass {
                label: _,
                color_attachments,
                depth_stencil_attachment,
            }
            | CommandListCommand::BeginRenderPassWithDiagnostics {
                label: _,
                color_attachments,
                depth_stencil_attachment,
                ..
            } => {
                validate_raster_queue(queue_class, "begin_render_pass")?;
                if active_compute_pass {
                    return Err(RhiError::InvalidComputePass {
                        reason:
                            "begin_render_pass cannot be recorded inside an active compute pass"
                                .to_string(),
                    });
                }
                if active_render_pass.is_some() {
                    return Err(RhiError::InvalidRenderPass {
                        reason: "render pass is already active".to_string(),
                    });
                }
                let attachment_info = validate_render_pass_attachments(
                    state,
                    color_attachments,
                    *depth_stencil_attachment,
                )?;
                active_render_pass = Some(ActiveRenderPass::new(
                    color_attachments,
                    *depth_stencil_attachment,
                    attachment_info,
                ));
            }
            CommandListCommand::EndRenderPass => {
                if debug_group_stack
                    .last()
                    .is_some_and(|scope| *scope == DebugGroupScope::RenderPass)
                {
                    return Err(RhiError::InvalidDebugMarker {
                        reason: "render pass ended with an active debug group".to_string(),
                    });
                }
                if active_render_pass.take().is_none() {
                    return Err(RhiError::InvalidRenderPass {
                        reason: "end_render_pass requires an active render pass".to_string(),
                    });
                }
            }
            CommandListCommand::BeginComputePass { label }
            | CommandListCommand::BeginComputePassWithDiagnostics { label, .. } => {
                validate_compute_queue(queue_class, "begin_compute_pass")?;
                if active_render_pass.is_some() {
                    return Err(RhiError::InvalidRenderPass {
                        reason:
                            "begin_compute_pass cannot be recorded inside an active render pass"
                                .to_string(),
                    });
                }
                if active_compute_pass {
                    return Err(RhiError::InvalidComputePass {
                        reason: "compute pass is already active".to_string(),
                    });
                }
                validate_compute_pass_label(label)?;
                active_compute_pass = true;
            }
            CommandListCommand::EndComputePass => {
                if debug_group_stack
                    .last()
                    .is_some_and(|scope| *scope == DebugGroupScope::ComputePass)
                {
                    return Err(RhiError::InvalidDebugMarker {
                        reason: "compute pass ended with an active debug group".to_string(),
                    });
                }
                if !active_compute_pass {
                    return Err(RhiError::InvalidComputePass {
                        reason: "end_compute_pass requires an active compute pass".to_string(),
                    });
                }
                active_compute_pass = false;
            }
            CommandListCommand::SetPipeline { pipeline } => {
                let pipeline_desc = state
                    .pipelines
                    .get(pipeline)
                    .ok_or(RhiError::UnknownPipeline(pipeline.diagnostic_id()))?;
                render_state.set_pipeline(*pipeline, pipeline_desc);
            }
            CommandListCommand::SetBindGroup {
                slot,
                bind_group,
                dynamic_offsets,
            } => {
                let bind_group_desc = state.bind_group_desc_ref(*bind_group)?;
                validate_bind_group_desc(state, bind_group_desc)?;
                validate_bind_group_dynamic_offsets(
                    state,
                    *bind_group,
                    bind_group_desc,
                    dynamic_offsets,
                    limits,
                )?;
                if let Some((_pipeline, _kind, pipeline_desc)) = render_state.current_pipeline {
                    validate_bind_group_slot(
                        state,
                        pipeline_desc,
                        *slot,
                        *bind_group,
                        bind_group_desc,
                    )?;
                }
                render_state.set_bind_group(*slot, *bind_group);
            }
            CommandListCommand::SetViewport { viewport } => {
                let render_pass = require_active_render_pass(&active_render_pass, "set_viewport")?;
                render_pass.validate_viewport(*viewport)?;
            }
            CommandListCommand::SetScissorRect { rect } => {
                let render_pass =
                    require_active_render_pass(&active_render_pass, "set_scissor_rect")?;
                render_pass.validate_scissor_rect(*rect)?;
            }
            CommandListCommand::SetVertexBuffer {
                slot,
                buffer,
                offset,
                size,
            } => {
                let desc = state.buffer_desc(*buffer)?;
                ensure_buffer_usage(buffer.diagnostic_id(), desc, BufferUsage::VERTEX)?;
                ensure_binding_range(*buffer, desc, *offset, *size)?;
                render_state.set_vertex_buffer(*slot, *size);
            }
            CommandListCommand::SetIndexBuffer {
                buffer,
                offset,
                size,
                format,
            } => {
                let desc = state.buffer_desc(*buffer)?;
                ensure_buffer_usage(buffer.diagnostic_id(), desc, BufferUsage::INDEX)?;
                ensure_binding_range(*buffer, desc, *offset, *size)?;
                if *size < format.size_bytes() {
                    return Err(RhiError::InvalidRasterDraw {
                        reason: "index buffer binding must contain at least one index".to_string(),
                    });
                }
                if *size % format.size_bytes() != 0 {
                    return Err(RhiError::InvalidRasterDraw {
                        reason: format!(
                            "index buffer binding size must be aligned to {:?}",
                            format
                        ),
                    });
                }
                render_state.index_buffer = Some(IndexBufferBinding {
                    size_bytes: *size,
                    format: *format,
                });
            }
            CommandListCommand::Draw {
                vertex_start,
                vertex_count,
                instance_start,
                instance_count,
            } => {
                validate_raster_queue(queue_class, "draw")?;
                ensure_non_zero_draw_counts(*vertex_count, *instance_count)?;
                let pipeline = render_state.require_raster_pipeline()?;
                let render_pass = require_active_render_pass(&active_render_pass, "draw")?;
                render_pass.validate_pipeline_attachments(state, pipeline)?;
                render_state.ensure_required_bind_groups(state, pipeline, "draw")?;
                render_state.ensure_required_vertex_buffers(pipeline)?;
                render_state.validate_vertex_ranges(
                    pipeline,
                    *vertex_start,
                    *vertex_count,
                    *instance_start,
                    *instance_count,
                    true,
                )?;
            }
            CommandListCommand::DrawIndexed {
                index_start,
                index_count,
                base_vertex: _,
                instance_start,
                instance_count,
            } => {
                validate_raster_queue(queue_class, "draw_indexed")?;
                ensure_non_zero_draw_counts(*index_count, *instance_count)?;
                let pipeline = render_state.require_raster_pipeline()?;
                let render_pass = require_active_render_pass(&active_render_pass, "draw_indexed")?;
                render_pass.validate_pipeline_attachments(state, pipeline)?;
                render_state.ensure_required_bind_groups(state, pipeline, "draw_indexed")?;
                render_state.ensure_required_vertex_buffers(pipeline)?;
                let index_buffer =
                    render_state
                        .index_buffer
                        .ok_or_else(|| RhiError::InvalidRasterDraw {
                            reason: "draw_indexed requires a bound index buffer".to_string(),
                        })?;
                validate_index_range(index_buffer, *index_start, *index_count)?;
                render_state.validate_vertex_ranges(
                    pipeline,
                    0,
                    0,
                    *instance_start,
                    *instance_count,
                    false,
                )?;
            }
            CommandListCommand::DrawIndirect { arguments, offset } => {
                validate_indirect_raster_draw(
                    state,
                    &render_state,
                    &active_render_pass,
                    queue_class,
                    *arguments,
                    *offset,
                    1,
                    IndirectArgumentKind::Draw,
                    "draw_indirect",
                    false,
                )?;
            }
            CommandListCommand::DrawIndexedIndirect { arguments, offset } => {
                validate_indirect_raster_draw(
                    state,
                    &render_state,
                    &active_render_pass,
                    queue_class,
                    *arguments,
                    *offset,
                    1,
                    IndirectArgumentKind::IndexedDraw,
                    "draw_indexed_indirect",
                    true,
                )?;
            }
            CommandListCommand::MultiDrawIndirect {
                arguments,
                offset,
                count,
            } => {
                validate_indirect_raster_draw(
                    state,
                    &render_state,
                    &active_render_pass,
                    queue_class,
                    *arguments,
                    *offset,
                    *count,
                    IndirectArgumentKind::Draw,
                    "multi_draw_indirect",
                    false,
                )?;
            }
            CommandListCommand::MultiDrawIndexedIndirect {
                arguments,
                offset,
                count,
            } => {
                validate_indirect_raster_draw(
                    state,
                    &render_state,
                    &active_render_pass,
                    queue_class,
                    *arguments,
                    *offset,
                    *count,
                    IndirectArgumentKind::IndexedDraw,
                    "multi_draw_indexed_indirect",
                    true,
                )?;
            }
            CommandListCommand::MultiDrawIndirectCount {
                arguments,
                offset,
                count_buffer,
                count_offset,
                max_count,
            } => {
                validate_indirect_raster_draw(
                    state,
                    &render_state,
                    &active_render_pass,
                    queue_class,
                    *arguments,
                    *offset,
                    *max_count,
                    IndirectArgumentKind::Draw,
                    "multi_draw_indirect_count",
                    false,
                )?;
                validate_indirect_count_buffer(
                    *count_buffer,
                    state.buffer_desc(*count_buffer)?,
                    *count_offset,
                )?;
            }
            CommandListCommand::MultiDrawIndexedIndirectCount {
                arguments,
                offset,
                count_buffer,
                count_offset,
                max_count,
            } => {
                validate_indirect_raster_draw(
                    state,
                    &render_state,
                    &active_render_pass,
                    queue_class,
                    *arguments,
                    *offset,
                    *max_count,
                    IndirectArgumentKind::IndexedDraw,
                    "multi_draw_indexed_indirect_count",
                    true,
                )?;
                validate_indirect_count_buffer(
                    *count_buffer,
                    state.buffer_desc(*count_buffer)?,
                    *count_offset,
                )?;
            }
            CommandListCommand::DispatchCompute { x, y, z } => {
                ensure_no_active_render_pass(&active_render_pass, "dispatch_compute")?;
                validate_compute_queue(queue_class, "dispatch_compute")?;
                validate_compute_dispatch_counts(*x, *y, *z)?;
                let pipeline = render_state.require_compute_pipeline()?;
                render_state.ensure_required_bind_groups(state, pipeline, "dispatch_compute")?;
            }
            CommandListCommand::DispatchComputeIndirect { arguments, offset } => {
                ensure_no_active_render_pass(&active_render_pass, "dispatch_compute_indirect")?;
                validate_compute_queue(queue_class, "dispatch_compute_indirect")?;
                validate_indirect_arguments(
                    *arguments,
                    state.buffer_desc(*arguments)?,
                    *offset,
                    1,
                    IndirectArgumentKind::ComputeDispatch,
                )?;
                let pipeline = render_state.require_compute_pipeline()?;
                render_state.ensure_required_bind_groups(
                    state,
                    pipeline,
                    "dispatch_compute_indirect",
                )?;
            }
        }
    }
    if active_render_pass.is_some() {
        return Err(RhiError::InvalidRenderPass {
            reason: "command list ended with an active render pass".to_string(),
        });
    }
    if active_compute_pass {
        return Err(RhiError::InvalidComputePass {
            reason: "command list ended with an active compute pass".to_string(),
        });
    }
    if !debug_group_stack.is_empty() {
        return Err(RhiError::InvalidDebugMarker {
            reason: "command list ended with an active debug group".to_string(),
        });
    }
    Ok(())
}

pub(super) fn execute_recorded_commands(
    state: &mut DeterministicRhiContractDeviceState,
    commands: &[CommandListCommand],
) -> Result<(), RhiError> {
    for command in commands {
        if copy_commands::execute(state, command)? {
            continue;
        }
        match command {
            CommandListCommand::DebugMarker { .. }
            | CommandListCommand::PushDebugGroup { .. }
            | CommandListCommand::PopDebugGroup => {}
            CommandListCommand::CopyBufferToBuffer { .. }
            | CommandListCommand::CopyBufferToTexture { .. }
            | CommandListCommand::CopyTextureToBuffer { .. }
            | CommandListCommand::CopyTextureToTexture { .. } => {
                unreachable!("copy commands are handled by command_validation::copy_commands")
            }
            CommandListCommand::SetPipeline { .. }
            | CommandListCommand::BeginRenderPass { .. }
            | CommandListCommand::BeginRenderPassWithDiagnostics { .. }
            | CommandListCommand::EndRenderPass
            | CommandListCommand::BeginComputePass { .. }
            | CommandListCommand::BeginComputePassWithDiagnostics { .. }
            | CommandListCommand::EndComputePass
            | CommandListCommand::SetBindGroup { .. }
            | CommandListCommand::SetViewport { .. }
            | CommandListCommand::SetScissorRect { .. }
            | CommandListCommand::SetVertexBuffer { .. }
            | CommandListCommand::SetIndexBuffer { .. }
            | CommandListCommand::Draw { .. }
            | CommandListCommand::DrawIndexed { .. }
            | CommandListCommand::DrawIndirect { .. }
            | CommandListCommand::DrawIndexedIndirect { .. }
            | CommandListCommand::MultiDrawIndirect { .. }
            | CommandListCommand::MultiDrawIndexedIndirect { .. }
            | CommandListCommand::MultiDrawIndirectCount { .. }
            | CommandListCommand::MultiDrawIndexedIndirectCount { .. }
            | CommandListCommand::DispatchCompute { .. }
            | CommandListCommand::DispatchComputeIndirect { .. } => {}
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_indirect_raster_draw(
    state: &DeterministicRhiContractDeviceState,
    render_state: &RecordedRenderState<'_>,
    active_render_pass: &Option<ActiveRenderPass>,
    queue_class: RenderQueueClass,
    arguments: zr_rhi::BufferHandle,
    offset: u64,
    count: u32,
    kind: IndirectArgumentKind,
    command: &str,
    indexed: bool,
) -> Result<(), RhiError> {
    validate_raster_queue(queue_class, command)?;
    validate_indirect_arguments(
        arguments,
        state.buffer_desc(arguments)?,
        offset,
        count,
        kind,
    )?;
    let pipeline = render_state.require_raster_pipeline()?;
    let render_pass = require_active_render_pass(active_render_pass, command)?;
    render_pass.validate_pipeline_attachments(state, pipeline)?;
    render_state.ensure_required_bind_groups(state, pipeline, command)?;
    render_state.ensure_required_vertex_buffers(pipeline)?;
    if indexed && render_state.index_buffer.is_none() {
        return Err(RhiError::InvalidRasterDraw {
            reason: format!("{command} requires a bound index buffer"),
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DebugGroupScope {
    CommandEncoder,
    RenderPass,
    ComputePass,
}

impl DebugGroupScope {
    const fn from_active_scopes(in_render_pass: bool, in_compute_pass: bool) -> Self {
        if in_render_pass {
            Self::RenderPass
        } else if in_compute_pass {
            Self::ComputePass
        } else {
            Self::CommandEncoder
        }
    }
}

fn validate_debug_label(label: &str, role: &str) -> Result<(), RhiError> {
    if label.is_empty() {
        return Err(RhiError::InvalidDebugMarker {
            reason: format!("{role} label must not be empty"),
        });
    }
    Ok(())
}

fn validate_raster_queue(queue_class: RenderQueueClass, command: &str) -> Result<(), RhiError> {
    if queue_class == RenderQueueClass::Graphics {
        Ok(())
    } else {
        Err(RhiError::InvalidCommandQueue {
            queue: queue_class,
            command: command.to_string(),
        })
    }
}

fn require_active_render_pass<'a>(
    active_render_pass: &'a Option<ActiveRenderPass>,
    command: &str,
) -> Result<&'a ActiveRenderPass, RhiError> {
    active_render_pass
        .as_ref()
        .ok_or_else(|| RhiError::InvalidRenderPass {
            reason: format!("{command} requires an active render pass"),
        })
}

fn ensure_no_active_render_pass(
    active_render_pass: &Option<ActiveRenderPass>,
    command: &str,
) -> Result<(), RhiError> {
    if active_render_pass.is_some() {
        Err(RhiError::InvalidRenderPass {
            reason: format!("{command} cannot be recorded inside an active render pass"),
        })
    } else {
        Ok(())
    }
}

fn validate_compute_queue(queue_class: RenderQueueClass, command: &str) -> Result<(), RhiError> {
    if queue_class == RenderQueueClass::Copy {
        Err(RhiError::InvalidCommandQueue {
            queue: queue_class,
            command: command.to_string(),
        })
    } else {
        Ok(())
    }
}

fn validate_compute_pass_label(label: &str) -> Result<(), RhiError> {
    if label.is_empty() {
        Err(RhiError::InvalidComputePass {
            reason: "compute pass label must not be empty".to_string(),
        })
    } else {
        Ok(())
    }
}

fn validate_compute_dispatch_counts(x: u32, y: u32, z: u32) -> Result<(), RhiError> {
    if x == 0 || y == 0 || z == 0 {
        Err(RhiError::InvalidComputeDispatch {
            reason: "workgroup counts must be greater than zero".to_string(),
        })
    } else {
        Ok(())
    }
}

fn ensure_non_zero_draw_counts(draw_count: u32, instance_count: u32) -> Result<(), RhiError> {
    if draw_count == 0 || instance_count == 0 {
        Err(RhiError::InvalidRasterDraw {
            reason: "draw and instance counts must be greater than zero".to_string(),
        })
    } else {
        Ok(())
    }
}

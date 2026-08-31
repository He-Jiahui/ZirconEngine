use zr_rhi::{
    CommandListCommand, DiagnosticPassQueryScope, PipelineKind, RenderQueueClass, RhiError,
};

use super::super::diagnostics::WgpuDiagnosticQueryFrame;
use super::super::registry::WgpuResourceRegistry;
use super::compute::{
    encode_compute_dispatch_indirect_into_pass, encode_compute_dispatch_into_pass,
};
use super::state::{
    bind_groups_compute, pop_debug_group, validate_bind_group_slot, validate_debug_label,
    DebugGroupScope, EncoderState,
};
use crate::bind_group_validation::validate_bind_group_dynamic_offsets;

/// Encodes one explicit neutral compute pass. Its scope is retained so a
/// graph pass can contain several dispatches without recreating a native pass.
#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_compute_pass(
    encoder: &mut wgpu::CommandEncoder,
    registry: &WgpuResourceRegistry,
    commands: &[CommandListCommand],
    begin_index: usize,
    label: &str,
    state: &mut EncoderState,
    debug_groups: &mut Vec<DebugGroupScope>,
    queue_class: RenderQueueClass,
    diagnostic_scope: Option<(&WgpuDiagnosticQueryFrame, DiagnosticPassQueryScope)>,
    limits: &zr_rhi::RenderDeviceLimits,
) -> Result<usize, RhiError> {
    require_compute_queue(queue_class, "begin_compute_pass")?;
    validate_compute_pass_label(label)?;
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some(label),
        timestamp_writes: None,
    });
    let pipeline_statistics_scope =
        diagnostic_scope.and_then(|(_, scope)| scope.pipeline_statistics());
    if let Some(scope) = pipeline_statistics_scope {
        let query_set = diagnostic_scope
            .and_then(|(frame, _)| frame.pipeline_statistics_query_set())
            .ok_or(RhiError::DiagnosticQueryPlanRequired)?;
        pass.begin_pipeline_statistics_query(query_set, scope.query_index());
    }
    let mut command_index = begin_index + 1;
    while command_index < commands.len() {
        match &commands[command_index] {
            CommandListCommand::EndComputePass => {
                if debug_groups
                    .last()
                    .is_some_and(|scope| *scope == DebugGroupScope::ComputePass)
                {
                    return Err(RhiError::InvalidDebugMarker {
                        reason: "compute pass ended with an active debug group".to_string(),
                    });
                }
                if pipeline_statistics_scope.is_some() {
                    pass.end_pipeline_statistics_query();
                }
                return Ok(command_index + 1);
            }
            CommandListCommand::BeginComputePass { .. }
            | CommandListCommand::BeginComputePassWithDiagnostics { .. } => {
                return Err(RhiError::InvalidComputePass {
                    reason: "compute pass is already active".to_string(),
                });
            }
            CommandListCommand::BeginRenderPass { .. }
            | CommandListCommand::BeginRenderPassWithDiagnostics { .. }
            | CommandListCommand::EndRenderPass
            | CommandListCommand::CopyBufferToBuffer { .. }
            | CommandListCommand::CopyBufferToTexture { .. }
            | CommandListCommand::CopyTextureToBuffer { .. }
            | CommandListCommand::CopyTextureToTexture { .. }
            | CommandListCommand::Draw { .. }
            | CommandListCommand::DrawIndexed { .. }
            | CommandListCommand::DrawIndirect { .. }
            | CommandListCommand::DrawIndexedIndirect { .. }
            | CommandListCommand::MultiDrawIndirect { .. }
            | CommandListCommand::MultiDrawIndexedIndirect { .. }
            | CommandListCommand::MultiDrawIndirectCount { .. }
            | CommandListCommand::MultiDrawIndexedIndirectCount { .. }
            | CommandListCommand::SetViewport { .. }
            | CommandListCommand::SetScissorRect { .. }
            | CommandListCommand::SetVertexBuffer { .. }
            | CommandListCommand::SetIndexBuffer { .. } => {
                return Err(RhiError::InvalidComputePass {
                    reason: "render, copy, and raster-state commands cannot be recorded inside an active compute pass"
                        .to_string(),
                });
            }
            CommandListCommand::DebugMarker { label } => {
                validate_debug_label(label, "debug marker")?;
                pass.insert_debug_marker(label);
            }
            CommandListCommand::PushDebugGroup { label } => {
                validate_debug_label(label, "debug group")?;
                pass.push_debug_group(label);
                debug_groups.push(DebugGroupScope::ComputePass);
            }
            CommandListCommand::PopDebugGroup => {
                pop_debug_group(debug_groups, DebugGroupScope::ComputePass)?;
                pass.pop_debug_group();
            }
            CommandListCommand::SetPipeline { pipeline } => {
                state.set_pipeline(*pipeline, registry.pipeline_desc(*pipeline)?);
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
            }
            CommandListCommand::DispatchCompute { x, y, z } => {
                encode_compute_dispatch_into_pass(
                    &mut pass,
                    registry,
                    state,
                    queue_class,
                    *x,
                    *y,
                    *z,
                )?;
            }
            CommandListCommand::DispatchComputeIndirect { arguments, offset } => {
                encode_compute_dispatch_indirect_into_pass(
                    &mut pass,
                    registry,
                    state,
                    queue_class,
                    *arguments,
                    *offset,
                )?;
            }
        }
        command_index += 1;
    }

    Err(RhiError::InvalidComputePass {
        reason: "command list ended with an active compute pass".to_string(),
    })
}

fn require_compute_queue(queue_class: RenderQueueClass, command: &str) -> Result<(), RhiError> {
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

use zr_rhi::{BufferHandle, BufferUsage, PipelineKind, RenderQueueClass, RhiError};

use crate::indirect_validation::{validate_indirect_arguments, IndirectArgumentKind};
use crate::resource_validation::ensure_buffer_usage;

use super::super::registry::WgpuResourceRegistry;
use super::state::{bind_groups_compute, validate_draw_bindings, EncoderState};

/// Encodes the M2 buffer-copy and compute subset outside render passes.
pub(crate) fn encode_buffer_copy(
    encoder: &mut wgpu::CommandEncoder,
    registry: &WgpuResourceRegistry,
    source: BufferHandle,
    destination: BufferHandle,
    source_offset: u64,
    destination_offset: u64,
    size: u64,
) -> Result<(), RhiError> {
    if source == destination {
        return Err(RhiError::InvalidCopy {
            reason: "buffer copies must use distinct source and destination buffers".to_string(),
        });
    }
    if size == 0 || source_offset % 4 != 0 || destination_offset % 4 != 0 || size % 4 != 0 {
        return Err(RhiError::InvalidCopy {
            reason: "WGPU buffer copy offsets and size must be non-zero multiples of four"
                .to_string(),
        });
    }
    let source_desc = registry.buffer_desc(source)?;
    let destination_desc = registry.buffer_desc(destination)?;
    ensure_buffer_usage(source.diagnostic_id(), &source_desc, BufferUsage::COPY_SRC)?;
    ensure_buffer_usage(
        destination.diagnostic_id(),
        &destination_desc,
        BufferUsage::COPY_DST,
    )?;
    if source_offset.saturating_add(size) > source_desc.size_bytes
        || destination_offset.saturating_add(size) > destination_desc.size_bytes
    {
        return Err(RhiError::BufferCopyOutOfRange {
            source_buffer: source.diagnostic_id(),
            destination_buffer: destination.diagnostic_id(),
            source_offset,
            destination_offset,
            size,
        });
    }
    encoder.copy_buffer_to_buffer(
        registry.buffer(source)?,
        source_offset,
        registry.buffer(destination)?,
        destination_offset,
        size,
    );
    Ok(())
}

pub(crate) fn encode_compute_dispatch(
    encoder: &mut wgpu::CommandEncoder,
    registry: &WgpuResourceRegistry,
    state: &EncoderState,
    queue_class: RenderQueueClass,
    x: u32,
    y: u32,
    z: u32,
) -> Result<(), RhiError> {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: state
            .current_pipeline
            .as_ref()
            .and_then(|(_, pipeline)| pipeline.label.as_deref()),
        timestamp_writes: None,
    });
    encode_compute_dispatch_into_pass(&mut pass, registry, state, queue_class, x, y, z)
}

pub(crate) fn encode_compute_dispatch_indirect(
    encoder: &mut wgpu::CommandEncoder,
    registry: &WgpuResourceRegistry,
    state: &EncoderState,
    queue_class: RenderQueueClass,
    arguments: BufferHandle,
    offset: u64,
) -> Result<(), RhiError> {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: state
            .current_pipeline
            .as_ref()
            .and_then(|(_, pipeline)| pipeline.label.as_deref()),
        timestamp_writes: None,
    });
    encode_compute_dispatch_indirect_into_pass(
        &mut pass,
        registry,
        state,
        queue_class,
        arguments,
        offset,
    )
}

pub(crate) fn encode_compute_dispatch_into_pass(
    pass: &mut wgpu::ComputePass<'_>,
    registry: &WgpuResourceRegistry,
    state: &EncoderState,
    queue_class: RenderQueueClass,
    x: u32,
    y: u32,
    z: u32,
) -> Result<(), RhiError> {
    validate_compute_dispatch(queue_class, x, y, z)?;
    let (pipeline_handle, pipeline) = state.require_pipeline(PipelineKind::Compute)?;
    validate_draw_bindings(registry, state, pipeline, "dispatch_compute")?;
    pass.set_pipeline(registry.compute_pipeline(pipeline_handle)?);
    bind_groups_compute(pass, registry, state, pipeline)?;
    pass.dispatch_workgroups(x, y, z);
    Ok(())
}

pub(crate) fn encode_compute_dispatch_indirect_into_pass(
    pass: &mut wgpu::ComputePass<'_>,
    registry: &WgpuResourceRegistry,
    state: &EncoderState,
    queue_class: RenderQueueClass,
    arguments: BufferHandle,
    offset: u64,
) -> Result<(), RhiError> {
    if queue_class == RenderQueueClass::Copy {
        return Err(RhiError::InvalidCommandQueue {
            queue: queue_class,
            command: "dispatch_compute_indirect".to_string(),
        });
    }
    validate_indirect_arguments(
        arguments,
        &registry.buffer_desc(arguments)?,
        offset,
        1,
        IndirectArgumentKind::ComputeDispatch,
    )?;
    let (pipeline_handle, pipeline) = state.require_pipeline(PipelineKind::Compute)?;
    validate_draw_bindings(registry, state, pipeline, "dispatch_compute_indirect")?;
    pass.set_pipeline(registry.compute_pipeline(pipeline_handle)?);
    bind_groups_compute(pass, registry, state, pipeline)?;
    pass.dispatch_workgroups_indirect(registry.buffer(arguments)?, offset);
    Ok(())
}

fn validate_compute_dispatch(
    queue_class: RenderQueueClass,
    x: u32,
    y: u32,
    z: u32,
) -> Result<(), RhiError> {
    if queue_class == RenderQueueClass::Copy {
        return Err(RhiError::InvalidCommandQueue {
            queue: queue_class,
            command: "dispatch_compute".to_string(),
        });
    }
    if x == 0 || y == 0 || z == 0 {
        return Err(RhiError::InvalidComputeDispatch {
            reason: "workgroup counts must be greater than zero".to_string(),
        });
    }
    Ok(())
}

use zr_rhi::{BufferDesc, BufferHandle, BufferUsage, RhiError};

use crate::resource_validation::ensure_buffer_usage;

const INDIRECT_ARGUMENT_ALIGNMENT_BYTES: u64 = 4;
const DRAW_INDIRECT_ARGUMENT_BYTES: u64 = 16;
const DRAW_INDEXED_INDIRECT_ARGUMENT_BYTES: u64 = 20;
const DISPATCH_INDIRECT_ARGUMENT_BYTES: u64 = 12;
const INDIRECT_COUNT_BYTES: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IndirectArgumentKind {
    Draw,
    IndexedDraw,
    ComputeDispatch,
}

impl IndirectArgumentKind {
    const fn argument_size_bytes(self) -> u64 {
        match self {
            Self::Draw => DRAW_INDIRECT_ARGUMENT_BYTES,
            Self::IndexedDraw => DRAW_INDEXED_INDIRECT_ARGUMENT_BYTES,
            Self::ComputeDispatch => DISPATCH_INDIRECT_ARGUMENT_BYTES,
        }
    }

    fn range_error(self, reason: &'static str) -> RhiError {
        match self {
            Self::Draw | Self::IndexedDraw => RhiError::InvalidRasterDraw {
                reason: reason.to_string(),
            },
            Self::ComputeDispatch => RhiError::InvalidComputeDispatch {
                reason: reason.to_string(),
            },
        }
    }
}

/// Validates WGPU's byte-level ABI before either deterministic validation or
/// native encoding consumes an indirect argument buffer.
pub(crate) fn validate_indirect_arguments(
    arguments: BufferHandle,
    desc: &BufferDesc,
    offset: u64,
    count: u32,
    kind: IndirectArgumentKind,
) -> Result<(), RhiError> {
    ensure_buffer_usage(arguments.diagnostic_id(), desc, BufferUsage::INDIRECT)?;
    if offset % INDIRECT_ARGUMENT_ALIGNMENT_BYTES != 0 {
        return Err(kind.range_error("indirect argument offset must be a multiple of four"));
    }
    let Some(argument_bytes) = kind.argument_size_bytes().checked_mul(u64::from(count)) else {
        return Err(kind.range_error("indirect argument range exceeds buffer"));
    };
    let Some(range_end) = offset.checked_add(argument_bytes) else {
        return Err(kind.range_error("indirect argument range exceeds buffer"));
    };
    if range_end > desc.size_bytes {
        return Err(kind.range_error("indirect argument range exceeds buffer"));
    }
    Ok(())
}

/// Validates the one-word GPU-written count consumed by the optional WGPU
/// multi-draw-count commands.
pub(crate) fn validate_indirect_count_buffer(
    count_buffer: BufferHandle,
    desc: &BufferDesc,
    offset: u64,
) -> Result<(), RhiError> {
    ensure_buffer_usage(count_buffer.diagnostic_id(), desc, BufferUsage::INDIRECT)?;
    if offset % INDIRECT_ARGUMENT_ALIGNMENT_BYTES != 0 {
        return Err(RhiError::InvalidRasterDraw {
            reason: "indirect count offset must be a multiple of four".to_string(),
        });
    }
    if offset
        .checked_add(INDIRECT_COUNT_BYTES)
        .is_none_or(|range_end| range_end > desc.size_bytes)
    {
        return Err(RhiError::InvalidRasterDraw {
            reason: "indirect count range exceeds buffer".to_string(),
        });
    }
    Ok(())
}

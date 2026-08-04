use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReadbackTicket(u64);

impl ReadbackTicket {
    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }
}

pub type ReadbackCallback =
    Box<dyn for<'a> FnOnce(Result<&'a [u8], ReadbackError>) + Send + 'static>;

#[derive(Debug, thiserror::Error)]
pub enum ReadbackError {
    #[error("readback range {range:?} must be non-empty")]
    EmptyRange { range: Range<u64> },
    #[error(
        "readback byte length {byte_len} must be aligned to WGPU copy-buffer alignment {alignment}"
    )]
    UnalignedCopySize { byte_len: u64, alignment: u64 },
    #[error("readback frame {requested} cannot start while frame {active} is active")]
    FrameAlreadyActive { active: u64, requested: u64 },
    #[error("readback frame {requested} was not prepared before encoding or mapping")]
    FrameNotActive { requested: u64 },
    #[error("readback requests require an active prepared frame")]
    NoActiveFrame,
    #[error("readback frame {frame_index} was aborted before completion")]
    FrameAborted { frame_index: u64 },
    #[error("readback source offset {source_offset} is not aligned to {alignment}")]
    UnalignedSourceOffset { source_offset: u64, alignment: u64 },
    #[error("readback staging slot {slot_index} is busy; this frame was not admitted")]
    SlotReuseIncomplete { slot_index: usize },
    #[error("readback staging capacity overflowed")]
    CapacityOverflow,
    #[error("texture readback extent {width}x{height} is invalid")]
    InvalidTextureExtent { width: u32, height: u32 },
    #[error(
        "readback frame {frame_index} has no staging buffer in slot {slot_index} after encoding"
    )]
    StagingBufferUnavailable { slot_index: usize, frame_index: u64 },
    #[error("WGPU buffer mapping failed: {0}")]
    BufferMap(String),
}

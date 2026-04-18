pub use zircon_framework::render::FrameHistoryHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameHistorySlot {
    AmbientOcclusion,
    GlobalIllumination,
    SceneColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameHistoryAccess {
    Read,
    Write,
    ReadWrite,
}

impl FrameHistoryAccess {
    pub const fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::ReadWrite, _) | (_, Self::ReadWrite) => Self::ReadWrite,
            (Self::Read, Self::Write) | (Self::Write, Self::Read) => Self::ReadWrite,
            (Self::Read, Self::Read) => Self::Read,
            (Self::Write, Self::Write) => Self::Write,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameHistoryBinding {
    pub slot: FrameHistorySlot,
    pub access: FrameHistoryAccess,
}

impl FrameHistoryBinding {
    pub const fn read(slot: FrameHistorySlot) -> Self {
        Self {
            slot,
            access: FrameHistoryAccess::Read,
        }
    }

    pub const fn write(slot: FrameHistorySlot) -> Self {
        Self {
            slot,
            access: FrameHistoryAccess::Write,
        }
    }

    pub const fn read_write(slot: FrameHistorySlot) -> Self {
        Self {
            slot,
            access: FrameHistoryAccess::ReadWrite,
        }
    }
}

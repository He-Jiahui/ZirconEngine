use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer};
use crate::handles::ZrRuntimeViewportHandle;

use super::viewport::ZrRuntimeViewportSizeV1;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeHostFetchRequestV1 {
    pub abi_version: u32,
    pub uri: ZrByteSlice,
    pub flags: u32,
}

impl ZrRuntimeHostFetchRequestV1 {
    pub const fn new(abi_version: u32, uri: ZrByteSlice, flags: u32) -> Self {
        Self {
            abi_version,
            uri,
            flags,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeFrameRequestV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
}

impl ZrRuntimeFrameRequestV1 {
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeAccessibilityTreeRequestV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub generation_hint: u64,
}

impl ZrRuntimeAccessibilityTreeRequestV1 {
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        generation_hint: u64,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
            generation_hint,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZrRuntimeFrameV1 {
    pub abi_version: u32,
    pub width: u32,
    pub height: u32,
    pub generation: u64,
    pub rgba: ZrOwnedByteBuffer,
}

impl ZrRuntimeFrameV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            width: 0,
            height: 0,
            generation: 0,
            rgba: ZrOwnedByteBuffer::empty(),
        }
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0 || self.rgba.is_empty()
    }
}

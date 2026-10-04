use crate::buffer::{ZrByteSlice, ZrOwnedResultV2};
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
/// 同一视口的捕获与呈现请求；宿主传入目标尺寸，运行时负责生成对应帧产品。
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
#[derive(Debug)]
/// 捕获帧的输出参数；RGBA 缓冲区由运行时分配，宿主在成功和失败路径均按原会话释放。
pub struct ZrRuntimeFrameV2 {
    pub abi_version: u32,
    pub width: u32,
    pub height: u32,
    pub generation: u64,
    pub rgba: ZrOwnedResultV2,
}

impl ZrRuntimeFrameV2 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            width: 0,
            height: 0,
            generation: 0,
            rgba: ZrOwnedResultV2::empty(),
        }
    }

    /// Reports only the canonical cleared out-parameter state, never a malformed frame.
    /// 仅判断尺寸与 RGBA 输出是否为空；调用方仍须核对调用状态、ABI 版本和帧代数。
    pub const fn is_empty(&self) -> bool {
        self.width == 0 && self.height == 0 && self.rgba.is_empty()
    }
}

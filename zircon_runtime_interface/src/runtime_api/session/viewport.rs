use crate::handles::ZrRuntimeViewportHandle;

use super::super::constants::{
    ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1, ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeViewportSizeV1 {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeViewportMetricsV1 {
    pub logical_size: ZrRuntimeViewportSizeV1,
    pub device_scale_factor: f32,
    pub physical_size: ZrRuntimeViewportSizeV1,
}

impl ZrRuntimeViewportMetricsV1 {
    pub const fn new(
        logical_size: ZrRuntimeViewportSizeV1,
        device_scale_factor: f32,
        physical_size: ZrRuntimeViewportSizeV1,
    ) -> Self {
        Self {
            logical_size,
            device_scale_factor,
            physical_size,
        }
    }
}

impl ZrRuntimeViewportSizeV1 {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeNativeSurfaceTargetV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub window_handle: u64,
    pub display_handle: u64,
}

impl ZrRuntimeNativeSurfaceTargetV1 {
    pub const fn none(abi_version: u32) -> Self {
        Self {
            abi_version,
            kind: ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1,
            window_handle: 0,
            display_handle: 0,
        }
    }

    pub const fn win32(abi_version: u32, hwnd: u64, hinstance: u64) -> Self {
        Self {
            abi_version,
            kind: ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
            window_handle: hwnd,
            display_handle: hinstance,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeBindViewportSurfaceRequestV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub target: ZrRuntimeNativeSurfaceTargetV1,
}

impl ZrRuntimeBindViewportSurfaceRequestV1 {
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        target: ZrRuntimeNativeSurfaceTargetV1,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
            target,
        }
    }
}

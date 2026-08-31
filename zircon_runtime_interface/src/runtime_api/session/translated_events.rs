use crate::buffer::ZrByteSlice;
use crate::handles::ZrRuntimeViewportHandle;

use super::super::constants::*;
use super::events::ZrRuntimeEventV1;
use super::viewport::{ZrRuntimeViewportMetricsV1, ZrRuntimeViewportSizeV1};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeTranslatedEventV1 {
    pub abi_version: u32,
    pub event: ZrRuntimeEventV1,
    pub host_reason: ZrByteSlice,
}

impl ZrRuntimeTranslatedEventV1 {
    pub const fn new(abi_version: u32, event: ZrRuntimeEventV1, host_reason: ZrByteSlice) -> Self {
        Self {
            abi_version,
            event,
            host_reason,
        }
    }

    pub const fn viewport_metrics(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        metrics: ZrRuntimeViewportMetricsV1,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::viewport_metrics(abi_version, viewport, metrics),
            ZrByteSlice::from_static(b"viewport_metrics"),
        )
    }

    pub const fn touch_moved(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        pointer_id: u64,
        x: f32,
        y: f32,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::touch(
                abi_version,
                viewport,
                pointer_id,
                ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
                x,
                y,
            ),
            ZrByteSlice::from_static(b"touch_moved"),
        )
    }

    pub const fn keyboard_text(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        key_text: ZrByteSlice,
    ) -> Self {
        Self::new(
            abi_version,
            ZrRuntimeEventV1::keyboard(
                abi_version,
                viewport,
                ZR_RUNTIME_KEY_ACTION_TEXT_V1,
                0,
                0,
                key_text,
            ),
            ZrByteSlice::from_static(b"keyboard_text"),
        )
    }
}

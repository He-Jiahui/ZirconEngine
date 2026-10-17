//! 宿主翻译原因与标准事件的 Rust 包装，不是额外的 C 布局事件信封。
//! 原因和事件 payload 都是借用描述符；复制包装不延长源字节的生命周期。

use crate::buffer::ZrByteSlice;
use crate::handles::ZrRuntimeViewportHandle;

use super::super::constants::*;
use super::events::ZrRuntimeEventV1;
use super::viewport::ZrRuntimeViewportMetricsV1;

/// 将宿主侧的翻译原因与标准 ABI 事件成对传递；调用者仍按 event 的 kind 解码其负载。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeTranslatedEventV1 {
    /// 包装版本原样保存，构造器不核对其与内部 event.abi_version 的关系。
    pub abi_version: u32,
    /// 按内部 kind 解读的标准事件；其 payload 仍借用原 owner。
    pub event: ZrRuntimeEventV1,
    /// 翻译原因的字节借用；便捷构造器使用静态原因，一般构造器不复制或校验。
    pub host_reason: ZrByteSlice,
}

impl ZrRuntimeTranslatedEventV1 {
    /// 保存三个参数并返回包装，不验证版本或原因编码；调用方须保持两处借用的源字节有效。
    pub const fn new(abi_version: u32, event: ZrRuntimeEventV1, host_reason: ZrByteSlice) -> Self {
        Self {
            abi_version,
            event,
            host_reason,
        }
    }

    /// 构造视口度量事件并附静态翻译原因；size 使用 metrics 的物理尺寸，构造不执行派发。
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

    /// 将 pointer_id 与 x/y 按值写入移动触点事件，并附静态翻译原因；不验证触点生命周期。
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

    /// 构造纯文本键动作并清零两键码，附静态翻译原因；key_text 的字节借用不因包装而延长。
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

use serde::{Deserialize, Serialize};

use crate::ui::{layout::UiFrame, tree::UiDirtyFlags};

/// 指针处理器每次只返回一个派发决议：Handled、捕获和焦点效果结束当前路由；
/// Blocked 在预览阶段阻止后续路由，在目标或冒泡阶段仅截断当前命中候选；
/// Passthrough 保留透传记录并继续，脏标记及伤害请求先累积到结果。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum UiPointerDispatchEffect {
    #[default]
    Unhandled,
    Handled,
    Blocked,
    Passthrough,
    CapturePointer,
    ReleasePointerCapture,
    SetFocus,
    ClearFocus,
    RequestDirty(UiDirtyFlags),
    // TODO: [CR-DISPATCH-0004] Runtime 目前只将帧收集到 requested_damage；
    // 统一输入适配器没有转发，需确认直接派发以外的渲染消费契约。
    RequestDamage(UiFrame),
}

impl UiPointerDispatchEffect {
    pub const fn handled() -> Self {
        Self::Handled
    }

    pub const fn blocked() -> Self {
        Self::Blocked
    }

    pub const fn passthrough() -> Self {
        Self::Passthrough
    }

    pub const fn capture() -> Self {
        Self::CapturePointer
    }

    pub const fn release_capture() -> Self {
        Self::ReleasePointerCapture
    }

    pub const fn set_focus() -> Self {
        Self::SetFocus
    }

    pub const fn clear_focus() -> Self {
        Self::ClearFocus
    }

    pub const fn request_dirty(flags: UiDirtyFlags) -> Self {
        Self::RequestDirty(flags)
    }

    pub const fn request_damage(frame: UiFrame) -> Self {
        Self::RequestDamage(frame)
    }
}

use serde::{Deserialize, Serialize};

use crate::ui::{layout::UiFrame, tree::UiDirtyFlags};

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

use zircon_runtime::rhi::{UiSurfaceImageUvRect, UiSurfaceRect};

use super::super::ChromeImageUvRect;
use crate::ui::retained_host::host_contract::data::FrameRect;

pub(super) fn ui_rect(frame: &FrameRect) -> UiSurfaceRect {
    UiSurfaceRect::new(frame.x, frame.y, frame.width, frame.height)
}

pub(super) fn ui_image_uv_rect(rect: ChromeImageUvRect) -> UiSurfaceImageUvRect {
    UiSurfaceImageUvRect {
        min: rect.min,
        max: rect.max,
    }
}

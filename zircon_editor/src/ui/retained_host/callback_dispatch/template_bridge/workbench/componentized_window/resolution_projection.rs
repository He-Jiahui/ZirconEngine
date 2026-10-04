use zircon_runtime_interface::ui::layout::UiFrame;

// 指针与帧只在宿主和模板的边界转换一次；无效比例退回一倍以保持坐标可用。
pub(super) fn normalized_presentation_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

pub(in super::super) fn logical_axis_from_physical(
    physical_value: f32,
    physical_origin: f32,
    scale_factor: f32,
) -> f32 {
    (physical_value - physical_origin) / normalized_presentation_scale_factor(scale_factor)
}

pub(super) fn scale_frame(frame: UiFrame, scale_factor: f32) -> UiFrame {
    let scale_factor = normalized_presentation_scale_factor(scale_factor);
    UiFrame::new(
        frame.x * scale_factor,
        frame.y * scale_factor,
        frame.width * scale_factor,
        frame.height * scale_factor,
    )
}

#[cfg(test)]
#[path = "tests/resolution_projection.rs"]
mod tests;

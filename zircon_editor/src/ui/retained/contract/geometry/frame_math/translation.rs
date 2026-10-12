use super::super::super::super::data::{FrameRect, TemplateNodeFrameData};

pub(in crate::ui::retained_host::host_contract) fn translated(
    frame: &FrameRect,
    origin_x: f32,
    origin_y: f32,
) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

pub(in crate::ui::retained_host::host_contract) fn translated_template_frame(
    frame: &TemplateNodeFrameData,
    origin_x: f32,
    origin_y: f32,
) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

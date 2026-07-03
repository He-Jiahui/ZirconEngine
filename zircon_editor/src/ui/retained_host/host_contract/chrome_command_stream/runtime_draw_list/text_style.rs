use zircon_runtime::rhi::UiSurfaceTextStyle;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRunPaintStyle};

use crate::ui::retained_host::host_contract::paint_text::{
    font_face_for_paint_style, font_request_for_face,
};

pub(super) fn ui_text_style(style: UiTextRunPaintStyle) -> UiSurfaceTextStyle {
    match (style.strong, style.emphasis) {
        (true, true) => UiSurfaceTextStyle::StrongEmphasis,
        (true, false) => UiSurfaceTextStyle::Strong,
        (false, true) => UiSurfaceTextStyle::Emphasis,
        (false, false) => UiSurfaceTextStyle::Regular,
    }
}

pub(super) fn ui_text_font_family(style: UiTextRunPaintStyle) -> String {
    let face = font_face_for_paint_style(style);
    font_request_for_face(face).family
}

pub(super) fn ui_text_font_weight(style: UiTextRunPaintStyle) -> u16 {
    let face = font_face_for_paint_style(style);
    UiResolvedStyle::normalized_font_weight(font_request_for_face(face).weight)
}

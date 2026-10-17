use zircon_runtime::rhi::UiSurfaceTextStyle;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRunPaintStyle};

use crate::ui::retained_host::host_contract::paint_text::{
    font_face_for_paint_style, HostRuntimeTextFaces,
};

pub(super) fn ui_text_style(style: UiTextRunPaintStyle) -> UiSurfaceTextStyle {
    match (style.strong, style.emphasis) {
        (true, true) => UiSurfaceTextStyle::StrongEmphasis,
        (true, false) => UiSurfaceTextStyle::Strong,
        (false, true) => UiSurfaceTextStyle::Emphasis,
        (false, false) => UiSurfaceTextStyle::Regular,
    }
}

pub(super) fn ui_text_font_family(
    style: UiTextRunPaintStyle,
    faces: &HostRuntimeTextFaces,
) -> String {
    let face = font_face_for_paint_style(style);
    faces.face(face).family.to_string()
}

pub(super) fn ui_text_font_weight(style: UiTextRunPaintStyle, faces: &HostRuntimeTextFaces) -> u16 {
    let face = font_face_for_paint_style(style);
    UiResolvedStyle::normalized_font_weight(faces.face(face).weight)
}

pub(super) fn ui_text_font_request(
    style: UiTextRunPaintStyle,
    request: Option<&crate::ui::retained_host::host_contract::paint_text::HostTextFontRequest>,
    faces: &mut Option<HostRuntimeTextFaces>,
) -> (String, u16) {
    if let Some(request) = request
        .filter(|request| !request.family.trim().is_empty() && (1..=1000).contains(&request.weight))
    {
        return (request.family.clone(), request.weight);
    }
    // Legacy commands must not inherit an outer authored request while lazily
    // capturing the runtime face snapshot. Keep this fallback explicitly
    // preference based, then restore the surrounding scope on return.
    let _legacy =
        crate::ui::retained_host::host_contract::paint_text::HostTextFontRequestScope::enter(None);
    let faces = faces.get_or_insert_with(
        crate::ui::retained_host::host_contract::paint_text::capture_runtime_text_faces,
    );
    (
        ui_text_font_family(style, faces),
        ui_text_font_weight(style, faces),
    )
}
pub(super) fn ui_text_style_for_request(
    style: UiTextRunPaintStyle,
    request: Option<&crate::ui::retained_host::host_contract::paint_text::HostTextFontRequest>,
) -> UiSurfaceTextStyle {
    // RHI Strong promotes weight to >=700. Authored weight already selects the face.
    let authored = request.is_some_and(|request| {
        !request.family.trim().is_empty() && (1..=1000).contains(&request.weight)
    });
    ui_text_style(UiTextRunPaintStyle {
        strong: style.strong && !authored,
        ..style
    })
}

#[path = "paint_text/blend.rs"]
mod blend;
#[path = "paint_text/clip.rs"]
mod clip;
#[path = "paint_text/draw.rs"]
mod draw;
#[path = "paint_text/font.rs"]
mod font;
#[path = "paint_text/layout_policy.rs"]
mod layout_policy;
#[path = "paint_text/sync.rs"]
mod sync;

pub(crate) use draw::TextPaintEvidenceScope;

pub(in crate::ui::retained_host::host_contract) use draw::draw_text;
pub(in crate::ui::retained_host::host_contract) use draw::{
    draw_text_with_size_and_style, draw_text_with_size_and_style_and_layout_policy,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use font::take_runtime_text_face_capture_count;
pub(in crate::ui::retained_host::host_contract) use font::{
    active_authored_font_request, authored_font_request_for_node, HostTextFontRequestScope,
    HostTextFontRequest, capture_runtime_text_faces, font_face_for_paint_style, font_request_for_face,
    runtime_font_family_for_face, runtime_text_style_for_face, HostRuntimeTextFaces,
    HostTextFontFace,
};
pub(crate) use font::{
    measure_runtime_text_width, measure_runtime_text_width_with_style,
    runtime_text_metrics_generation,
};
pub(in crate::ui::retained_host::host_contract) use layout_policy::HostTextLayoutPolicy;

#[cfg(test)]
#[path = "tests/paint_text.rs"]
mod tests;

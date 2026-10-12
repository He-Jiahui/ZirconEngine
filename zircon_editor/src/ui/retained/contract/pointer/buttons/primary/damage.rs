use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::union_optional_frames;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;

pub(super) fn hit_damage(
    cleared_text_input_frame: Option<&FrameRect>,
    hit: &TemplateNodePointerHit,
) -> FrameRect {
    union_optional_frames(cleared_text_input_frame.cloned(), Some(hit.frame.clone()))
        .unwrap_or_else(|| hit.frame.clone())
}

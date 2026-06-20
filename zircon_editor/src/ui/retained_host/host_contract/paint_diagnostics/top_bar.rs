use super::super::data::{FrameRect, HostWindowLayoutData, HostWindowPresentationData};
use super::visibility::diagnostic_visible_frame;

pub(in crate::ui::retained_host::host_contract) fn presentation_top_bar_frame(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    let layout = if has_visible_root_frame(&presentation.host_scene_data.layout) {
        &presentation.host_scene_data.layout
    } else {
        &presentation.host_layout
    };
    let top_bar_height =
        if layout.center_band_frame.y.is_finite() && layout.center_band_frame.y > 1.0 {
            layout.center_band_frame.y
        } else {
            fallback_top_bar_height(height)
        };
    top_bar_frame_with_height(width, top_bar_height)
}

fn top_bar_frame_with_height(width: u32, height: f32) -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height,
    }
}

fn fallback_top_bar_height(height: u32) -> f32 {
    38.0_f32.min(height as f32 * 0.25)
}

fn has_visible_root_frame(layout: &HostWindowLayoutData) -> bool {
    diagnostic_visible_frame(&layout.center_band_frame)
        || diagnostic_visible_frame(&layout.status_bar_frame)
        || diagnostic_visible_frame(&layout.document_region_frame)
        || diagnostic_visible_frame(&layout.viewport_content_frame)
}

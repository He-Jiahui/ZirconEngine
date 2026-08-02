mod model;
mod selection;

pub(in crate::ui::retained_host::host_contract) use self::model::{RootFrames, zero_origin};
use self::selection::selected_root_layout;
use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::paint_diagnostics::presentation_top_bar_frame;
use super::super::paint_geometry::frame_or;

pub(in crate::ui::retained_host::host_contract) fn resolve_root_frames(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> RootFrames {
    let layout = selected_root_layout(presentation);
    let top_bar = presentation_top_bar_frame(width, height, presentation);
    let top_bar_height = top_bar.height;
    let fallback_status_height = 24.0_f32.min(height as f32 * 0.2);
    let status_bar = frame_or(
        &layout.status_bar_frame,
        FrameRect {
            x: 0.0,
            y: (height as f32 - fallback_status_height).max(top_bar_height),
            width: width as f32,
            height: fallback_status_height,
        },
    );
    let center_band = frame_or(
        &layout.center_band_frame,
        FrameRect {
            x: 0.0,
            y: top_bar_height,
            width: width as f32,
            height: (status_bar.y - top_bar_height).max(1.0),
        },
    );
    let left_region = frame_or(
        &layout.left_region_frame,
        FrameRect {
            x: 0.0,
            y: center_band.y,
            width: (width as f32 * 0.22).min(260.0),
            height: center_band.height,
        },
    );
    let right_region = frame_or(&layout.right_region_frame, FrameRect::default());
    let bottom_region = frame_or(&layout.bottom_region_frame, FrameRect::default());
    let document_region = frame_or(
        &layout.document_region_frame,
        FrameRect {
            x: left_region.x + left_region.width,
            y: center_band.y,
            width: (width as f32 - left_region.width).max(1.0),
            height: center_band.height,
        },
    );
    let viewport_region = frame_or(
        &layout.viewport_content_frame,
        FrameRect {
            x: document_region.x + 16.0,
            y: document_region.y + 28.0,
            width: (document_region.width - 32.0).max(1.0),
            height: (document_region.height - 56.0).max(1.0),
        },
    );
    RootFrames {
        top_bar,
        center_band,
        status_bar,
        left_region,
        right_region,
        bottom_region,
        document_region,
        viewport_region,
    }
}

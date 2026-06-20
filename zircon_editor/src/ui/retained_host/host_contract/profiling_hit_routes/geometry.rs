use super::super::data::{FrameRect, HostSideDockSurfaceData};

pub(in crate::ui::retained_host::host_contract) fn side_dock_content_frame(
    dock: &HostSideDockSurfaceData,
) -> FrameRect {
    let panel_x = if dock.rail_before_panel {
        dock.region_frame.x + dock.rail_width_px
    } else {
        dock.region_frame.x
    };
    translated(&dock.content_frame, panel_x, dock.region_frame.y)
}

pub(in crate::ui::retained_host::host_contract) fn floating_window_content_frame(
    frame: &FrameRect,
    header: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: frame.x + 1.0,
        y: frame.y + header.height.max(0.0) + 1.0,
        width: (frame.width - 2.0).max(0.0),
        height: (frame.height - header.height.max(0.0) - 2.0).max(0.0),
    }
}

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

pub(in crate::ui::retained_host::host_contract) fn contains(
    frame: &FrameRect,
    x: f32,
    y: f32,
) -> bool {
    frame.width > 0.0
        && frame.height > 0.0
        && x >= frame.x
        && y >= frame.y
        && x < frame.x + frame.width
        && y < frame.y + frame.height
}

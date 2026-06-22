use super::super::super::super::data::{FrameRect, HostSideDockSurfaceData};

pub(super) struct SideDockFrames {
    pub(super) rail_origin: FrameRect,
    pub(super) panel_origin: FrameRect,
}

pub(super) fn side_dock_frames(dock: &HostSideDockSurfaceData) -> SideDockFrames {
    if dock.rail_before_panel {
        return SideDockFrames {
            rail_origin: FrameRect {
                x: dock.region_frame.x,
                y: dock.region_frame.y,
                width: dock.rail_width_px,
                height: dock.region_frame.height,
            },
            panel_origin: FrameRect {
                x: dock.region_frame.x + dock.rail_width_px,
                y: dock.region_frame.y,
                width: dock.panel_width_px,
                height: dock.region_frame.height,
            },
        };
    }
    SideDockFrames {
        rail_origin: FrameRect {
            x: dock.region_frame.x + dock.panel_width_px,
            y: dock.region_frame.y,
            width: dock.rail_width_px,
            height: dock.region_frame.height,
        },
        panel_origin: FrameRect {
            x: dock.region_frame.x,
            y: dock.region_frame.y,
            width: dock.panel_width_px,
            height: dock.region_frame.height,
        },
    }
}

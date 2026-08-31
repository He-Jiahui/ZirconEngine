use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::{layout::UiPoint, surface::UiSurfaceFrame};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::profiling_artifacts::UiProfileNamedFrame;

use super::super::super::frame_math::{
    frame_rect_center_point, intersect_frames, push_named_profile_frame,
};

pub(super) fn collect_surface_frame_control_nodes(
    kind: &str,
    surface: &str,
    origin: &FrameRect,
    surface_frame: &UiSurfaceFrame,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    for node in &surface_frame.arranged_tree.nodes {
        if !node.supports_pointer() {
            continue;
        }
        let Some(control_id) = node.control_id.as_deref() else {
            continue;
        };
        let frame = FrameRect {
            x: origin.x + node.frame.x,
            y: origin.y + node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        let clip = FrameRect {
            x: origin.x + node.clip_frame.x,
            y: origin.y + node.clip_frame.y,
            width: node.clip_frame.width,
            height: node.clip_frame.height,
        };
        let Some(effective_frame) = intersect_frames(&frame, &clip) else {
            continue;
        };
        let center = frame_rect_center_point(&effective_frame);
        let local_center = UiPoint::new(center.x - origin.x, center.y - origin.y);
        let route_is_top_hit = hit_test_surface_frame(surface_frame, local_center)
            .top_hit
            .and_then(|node_id| surface_frame.arranged_tree.get(node_id))
            .and_then(|hit_node| hit_node.control_id.as_deref())
            .is_some_and(|hit_control_id| hit_control_id == control_id);
        if !route_is_top_hit {
            continue;
        }
        push_named_profile_frame(
            out,
            format!("{kind}.{surface}.{control_id}"),
            kind,
            surface,
            effective_frame.into(),
            Some(clip.into()),
        );
    }
}

use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiPoint, surface::UiSurfaceFrame};

use super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) struct SurfaceFramePointerHit {
    pub(in crate::ui::retained_host::host_contract) node_id: UiNodeId,
    pub(in crate::ui::retained_host::host_contract) control_id: SharedString,
    pub(in crate::ui::retained_host::host_contract) control_frame: FrameRect,
}

pub(in crate::ui::retained_host::host_contract) fn hit_test_host_surface_frame(
    surface_frame: &UiSurfaceFrame,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<SurfaceFramePointerHit> {
    let local_point = UiPoint::new(x - origin.x, y - origin.y);
    let hit = hit_test_surface_frame(surface_frame, local_point);
    let node_id = hit.top_hit?;
    let node = surface_frame.arranged_tree.get(node_id)?;
    let control_id = node.control_id.as_ref()?;
    Some(SurfaceFramePointerHit {
        node_id,
        control_id: control_id.as_str().into(),
        control_frame: FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
    })
}

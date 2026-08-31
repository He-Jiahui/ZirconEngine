use zircon_runtime_interface::math::{UVec2, Vec2};
use zircon_runtime_interface::ui::layout::UiPoint;

use crate::scene::viewport::pointer::candidates::handle_candidate;
use crate::scene::viewport::pointer::runtime_picking_adapter::resolve_runtime_route_for_candidates;
use crate::scene::viewport::projection::ViewportProjectionContext;
use crate::scene::viewport::{HandleOverlayExtract, ViewportCameraSnapshot};

use super::ViewportPointerRoute;

pub(in crate::scene::viewport) fn local_handle_route(
    handles: &[HandleOverlayExtract],
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
    cursor: Vec2,
) -> Option<ViewportPointerRoute> {
    let projection = ViewportProjectionContext::new(camera, viewport);
    let mut candidates =
        Vec::with_capacity(handles.iter().map(|handle| handle.elements.len()).sum());
    for handle in handles {
        for element in &handle.elements {
            if let Some(candidate) = handle_candidate(handle.owner, element, &projection) {
                candidates.push(candidate);
            }
        }
    }
    resolve_runtime_route_for_candidates(&candidates, UiPoint::new(cursor.x, cursor.y))
}

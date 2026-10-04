//! Play gizmo 等世界中立入口以现有 Handle 值进行一次局部拾取，仍使用 Runtime 的统一解析器，结果只声明 owner 和轴。

use zircon_runtime_interface::math::{UVec2, Vec2};
use zircon_runtime_interface::ui::layout::UiPoint;

use crate::scene::viewport::pointer::candidates::handle_candidate;
use crate::scene::viewport::pointer::runtime_picking_adapter::resolve_runtime_route_for_candidates;
use crate::scene::viewport::projection::ViewportProjectionContext;
use crate::scene::viewport::{HandleOverlayExtract, ViewportCameraSnapshot};

use super::ViewportPointerRoute;

/// 世界中立拾取只回答 owner 与轴；调用方仍需验证活动 Play 实例、gateway 身份和对应事务许可。
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

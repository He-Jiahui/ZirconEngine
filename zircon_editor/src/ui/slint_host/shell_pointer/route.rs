use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{dispatch::UiPointerDispatchResult, event_ui::UiNodeId};

use crate::ui::slint_host::drawer_resize::HostResizeTargetGroup;
use crate::ui::slint_host::tab_drag::HostDragTargetGroup;
use crate::ui::workbench::layout::DockEdge;
use crate::ui::workbench::layout::MainPageId;

use super::node_ids::{
    DOCUMENT_EDGE_BOTTOM_NODE_ID, DOCUMENT_EDGE_LEFT_NODE_ID, DOCUMENT_EDGE_RIGHT_NODE_ID,
    DOCUMENT_EDGE_TOP_NODE_ID, DRAG_TARGET_BOTTOM_NODE_ID, DRAG_TARGET_DOCUMENT_NODE_ID,
    DRAG_TARGET_LEFT_NODE_ID, DRAG_TARGET_RIGHT_NODE_ID, RESIZE_TARGET_BOTTOM_NODE_ID,
    RESIZE_TARGET_LEFT_NODE_ID, RESIZE_TARGET_RIGHT_NODE_ID,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HostShellPointerRoute {
    DragTarget(HostDragTargetGroup),
    DocumentEdge(DockEdge),
    FloatingWindow(MainPageId),
    FloatingWindowEdge {
        window_id: MainPageId,
        edge: DockEdge,
    },
    Resize(HostResizeTargetGroup),
}

pub(super) fn drag_route_from_node(
    node_id: UiNodeId,
    drag_routes: &BTreeMap<UiNodeId, HostShellPointerRoute>,
) -> Option<HostShellPointerRoute> {
    match node_id {
        DRAG_TARGET_LEFT_NODE_ID => {
            Some(HostShellPointerRoute::DragTarget(HostDragTargetGroup::Left))
        }
        DRAG_TARGET_RIGHT_NODE_ID => Some(HostShellPointerRoute::DragTarget(
            HostDragTargetGroup::Right,
        )),
        DRAG_TARGET_BOTTOM_NODE_ID => Some(HostShellPointerRoute::DragTarget(
            HostDragTargetGroup::Bottom,
        )),
        DOCUMENT_EDGE_LEFT_NODE_ID => Some(HostShellPointerRoute::DocumentEdge(DockEdge::Left)),
        DOCUMENT_EDGE_RIGHT_NODE_ID => Some(HostShellPointerRoute::DocumentEdge(DockEdge::Right)),
        DOCUMENT_EDGE_TOP_NODE_ID => Some(HostShellPointerRoute::DocumentEdge(DockEdge::Top)),
        DOCUMENT_EDGE_BOTTOM_NODE_ID => Some(HostShellPointerRoute::DocumentEdge(DockEdge::Bottom)),
        DRAG_TARGET_DOCUMENT_NODE_ID => Some(HostShellPointerRoute::DragTarget(
            HostDragTargetGroup::Document,
        )),
        _ => drag_routes.get(&node_id).cloned(),
    }
}

pub(super) fn resize_group_from_dispatch(
    dispatch: &UiPointerDispatchResult,
) -> Option<HostResizeTargetGroup> {
    match dispatch.handled_by.or(dispatch.captured_by) {
        Some(RESIZE_TARGET_LEFT_NODE_ID) => Some(HostResizeTargetGroup::Left),
        Some(RESIZE_TARGET_RIGHT_NODE_ID) => Some(HostResizeTargetGroup::Right),
        Some(RESIZE_TARGET_BOTTOM_NODE_ID) => Some(HostResizeTargetGroup::Bottom),
        _ => None,
    }
}

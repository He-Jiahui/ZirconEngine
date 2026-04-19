use std::collections::BTreeMap;

use zircon_ui::{dispatch::UiPointerDispatchResult, event_ui::UiNodeId};

use crate::ui::slint_host::drawer_resize::WorkbenchResizeTargetGroup;
use crate::ui::slint_host::tab_drag::WorkbenchDragTargetGroup;
use crate::{DockEdge, MainPageId};

use super::node_ids::{
    DOCUMENT_EDGE_BOTTOM_NODE_ID, DOCUMENT_EDGE_LEFT_NODE_ID, DOCUMENT_EDGE_RIGHT_NODE_ID,
    DOCUMENT_EDGE_TOP_NODE_ID, DRAG_TARGET_BOTTOM_NODE_ID, DRAG_TARGET_DOCUMENT_NODE_ID,
    DRAG_TARGET_LEFT_NODE_ID, DRAG_TARGET_RIGHT_NODE_ID, RESIZE_TARGET_BOTTOM_NODE_ID,
    RESIZE_TARGET_LEFT_NODE_ID, RESIZE_TARGET_RIGHT_NODE_ID,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchShellPointerRoute {
    DragTarget(WorkbenchDragTargetGroup),
    DocumentEdge(DockEdge),
    FloatingWindow(MainPageId),
    FloatingWindowEdge {
        window_id: MainPageId,
        edge: DockEdge,
    },
    Resize(WorkbenchResizeTargetGroup),
}

pub(super) fn drag_route_from_node(
    node_id: UiNodeId,
    drag_routes: &BTreeMap<UiNodeId, WorkbenchShellPointerRoute>,
) -> Option<WorkbenchShellPointerRoute> {
    match node_id {
        DRAG_TARGET_LEFT_NODE_ID => Some(WorkbenchShellPointerRoute::DragTarget(
            WorkbenchDragTargetGroup::Left,
        )),
        DRAG_TARGET_RIGHT_NODE_ID => Some(WorkbenchShellPointerRoute::DragTarget(
            WorkbenchDragTargetGroup::Right,
        )),
        DRAG_TARGET_BOTTOM_NODE_ID => Some(WorkbenchShellPointerRoute::DragTarget(
            WorkbenchDragTargetGroup::Bottom,
        )),
        DOCUMENT_EDGE_LEFT_NODE_ID => {
            Some(WorkbenchShellPointerRoute::DocumentEdge(DockEdge::Left))
        }
        DOCUMENT_EDGE_RIGHT_NODE_ID => {
            Some(WorkbenchShellPointerRoute::DocumentEdge(DockEdge::Right))
        }
        DOCUMENT_EDGE_TOP_NODE_ID => Some(WorkbenchShellPointerRoute::DocumentEdge(DockEdge::Top)),
        DOCUMENT_EDGE_BOTTOM_NODE_ID => {
            Some(WorkbenchShellPointerRoute::DocumentEdge(DockEdge::Bottom))
        }
        DRAG_TARGET_DOCUMENT_NODE_ID => Some(WorkbenchShellPointerRoute::DragTarget(
            WorkbenchDragTargetGroup::Document,
        )),
        _ => drag_routes.get(&node_id).cloned(),
    }
}

pub(super) fn resize_group_from_dispatch(
    dispatch: &UiPointerDispatchResult,
) -> Option<WorkbenchResizeTargetGroup> {
    match dispatch.handled_by.or(dispatch.captured_by) {
        Some(RESIZE_TARGET_LEFT_NODE_ID) => Some(WorkbenchResizeTargetGroup::Left),
        Some(RESIZE_TARGET_RIGHT_NODE_ID) => Some(WorkbenchResizeTargetGroup::Right),
        Some(RESIZE_TARGET_BOTTOM_NODE_ID) => Some(WorkbenchResizeTargetGroup::Bottom),
        _ => None,
    }
}

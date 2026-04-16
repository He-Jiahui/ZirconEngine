use std::collections::BTreeMap;

use zircon_ui::{
    UiNodeId, UiPoint, UiPointerButton, UiPointerDispatcher, UiPointerEvent, UiPointerEventKind,
    UiSurface,
};

use crate::host::slint_host::drawer_resize::WorkbenchResizeTargetGroup;
use crate::host::slint_host::tab_drag::WorkbenchDragTargetGroup;
use crate::{FloatingWindowModel, ShellSizePx, WorkbenchShellGeometry};

use super::drag_surface::build_drag_surface;
use super::resize_surface::{build_resize_surface, update_resize_surface};
use super::route::{drag_route_from_node, resize_group_from_dispatch, WorkbenchShellPointerRoute};

pub(crate) struct WorkbenchShellPointerBridge {
    drag_surface: UiSurface,
    drag_dispatcher: UiPointerDispatcher,
    drag_routes: BTreeMap<UiNodeId, WorkbenchShellPointerRoute>,
    resize_surface: UiSurface,
    resize_dispatcher: UiPointerDispatcher,
}

impl Default for WorkbenchShellPointerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkbenchShellPointerBridge {
    pub(crate) fn new() -> Self {
        let (drag_surface, drag_dispatcher, drag_routes) = build_drag_surface(
            ShellSizePx::new(1.0, 1.0),
            &WorkbenchShellGeometry::default(),
            false,
            &[],
        );
        let (resize_surface, resize_dispatcher) = build_resize_surface();
        Self {
            drag_surface,
            drag_dispatcher,
            drag_routes,
            resize_surface,
            resize_dispatcher,
        }
    }

    pub(crate) fn update_layout(
        &mut self,
        root_size: ShellSizePx,
        geometry: &WorkbenchShellGeometry,
        drawers_visible: bool,
    ) {
        self.update_layout_with_floating_windows(root_size, geometry, drawers_visible, &[]);
    }

    pub(crate) fn update_layout_with_floating_windows(
        &mut self,
        root_size: ShellSizePx,
        geometry: &WorkbenchShellGeometry,
        drawers_visible: bool,
        floating_windows: &[FloatingWindowModel],
    ) {
        let (drag_surface, drag_dispatcher, drag_routes) =
            build_drag_surface(root_size, geometry, drawers_visible, floating_windows);
        self.drag_surface = drag_surface;
        self.drag_dispatcher = drag_dispatcher;
        self.drag_routes = drag_routes;
        update_resize_surface(&mut self.resize_surface, root_size, geometry);
    }

    pub(crate) fn drag_target_at(&mut self, point: UiPoint) -> Option<WorkbenchDragTargetGroup> {
        self.drag_route_at(point).and_then(|route| match route {
            WorkbenchShellPointerRoute::DragTarget(group) => Some(group),
            WorkbenchShellPointerRoute::DocumentEdge(_)
            | WorkbenchShellPointerRoute::FloatingWindow(_)
            | WorkbenchShellPointerRoute::FloatingWindowEdge { .. } => {
                Some(WorkbenchDragTargetGroup::Document)
            }
            WorkbenchShellPointerRoute::Resize(_) => None,
        })
    }

    pub(crate) fn drag_route_at(&mut self, point: UiPoint) -> Option<WorkbenchShellPointerRoute> {
        let dispatch = self
            .drag_surface
            .dispatch_pointer_event(
                &self.drag_dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Move, point),
            )
            .ok()?;

        dispatch
            .handled_by
            .and_then(|node_id| drag_route_from_node(node_id, &self.drag_routes))
    }

    #[cfg(test)]
    pub(crate) fn resize_target_at(
        &mut self,
        point: UiPoint,
    ) -> Option<WorkbenchResizeTargetGroup> {
        self.dispatch_resize_event(UiPointerEvent::new(UiPointerEventKind::Move, point))
    }

    pub(crate) fn begin_resize(&mut self, point: UiPoint) -> Option<WorkbenchShellPointerRoute> {
        self.dispatch_resize_event(
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .map(WorkbenchShellPointerRoute::Resize)
    }

    pub(crate) fn update_resize(&mut self, point: UiPoint) -> Option<WorkbenchResizeTargetGroup> {
        self.dispatch_resize_event(UiPointerEvent::new(UiPointerEventKind::Move, point))
    }

    pub(crate) fn finish_resize(&mut self, point: UiPoint) -> Option<WorkbenchResizeTargetGroup> {
        self.dispatch_resize_event(
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
    }

    fn dispatch_resize_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Option<WorkbenchResizeTargetGroup> {
        let dispatch = self
            .resize_surface
            .dispatch_pointer_event(&self.resize_dispatcher, event)
            .ok()?;
        resize_group_from_dispatch(&dispatch)
    }
}

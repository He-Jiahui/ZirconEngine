use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
};

#[cfg(test)]
use crate::ui::host::NativeWindowHostState;
use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::slint_host::drawer_resize::HostResizeTargetGroup;
#[cfg(test)]
use crate::ui::slint_host::floating_window_projection::build_floating_window_projection_bundle_from_windows;
use crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::slint_host::tab_drag::HostDragTargetGroup;
use crate::ui::workbench::autolayout::ShellSizePx;
#[cfg(test)]
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::model::FloatingWindowModel;

use super::drag_surface::build_drag_surface;
use super::resize_surface::{build_resize_surface, update_resize_surface};
use super::route::{drag_route_from_node, resize_group_from_dispatch, HostShellPointerRoute};

pub(crate) struct HostShellPointerBridge {
    drag_surface: UiSurface,
    drag_dispatcher: UiPointerDispatcher,
    drag_routes: BTreeMap<UiNodeId, HostShellPointerRoute>,
    resize_surface: UiSurface,
    resize_dispatcher: UiPointerDispatcher,
}

impl Default for HostShellPointerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl HostShellPointerBridge {
    pub(crate) fn new() -> Self {
        let (drag_surface, drag_dispatcher, drag_routes) = build_drag_surface(
            ShellSizePx::new(1.0, 1.0),
            &WorkbenchShellGeometry::default(),
            false,
            &[],
            None,
            None,
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

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn update_layout(
        &mut self,
        root_size: ShellSizePx,
        geometry: &WorkbenchShellGeometry,
        drawers_visible: bool,
    ) {
        self.update_layout_with_root_shell_frames(
            root_size,
            geometry,
            drawers_visible,
            &[],
            None,
            None,
        );
    }

    #[cfg(test)]
    pub(crate) fn update_layout_with_floating_windows(
        &mut self,
        root_size: ShellSizePx,
        geometry: &WorkbenchShellGeometry,
        drawers_visible: bool,
        floating_windows: &[FloatingWindowModel],
        _native_window_hosts: &[NativeWindowHostState],
    ) {
        self.update_layout_with_root_shell_frames(
            root_size,
            geometry,
            drawers_visible,
            floating_windows,
            None,
            None,
        );
    }

    #[cfg(test)]
    pub(crate) fn update_layout_with_native_window_hosts(
        &mut self,
        root_size: ShellSizePx,
        geometry: &WorkbenchShellGeometry,
        drawers_visible: bool,
        floating_windows: &[FloatingWindowModel],
        shared_root_frames: Option<&BuiltinHostRootShellFrames>,
        native_window_hosts: &[NativeWindowHostState],
    ) {
        let floating_window_projection_bundle =
            build_floating_window_projection_bundle_from_windows(
                floating_windows,
                geometry,
                &WorkbenchChromeMetrics::default(),
                native_window_hosts,
            );
        self.update_layout_with_root_shell_frames(
            root_size,
            geometry,
            drawers_visible,
            floating_windows,
            shared_root_frames,
            Some(&floating_window_projection_bundle),
        );
    }

    pub(crate) fn update_layout_with_root_shell_frames(
        &mut self,
        root_size: ShellSizePx,
        geometry: &WorkbenchShellGeometry,
        drawers_visible: bool,
        floating_windows: &[FloatingWindowModel],
        shared_root_frames: Option<&BuiltinHostRootShellFrames>,
        floating_window_projection_bundle: Option<&FloatingWindowProjectionBundle>,
    ) {
        let (drag_surface, drag_dispatcher, drag_routes) = build_drag_surface(
            root_size,
            geometry,
            drawers_visible,
            floating_windows,
            shared_root_frames,
            floating_window_projection_bundle,
        );
        self.drag_surface = drag_surface;
        self.drag_dispatcher = drag_dispatcher;
        self.drag_routes = drag_routes;
        update_resize_surface(&mut self.resize_surface, root_size, geometry);
    }

    pub(crate) fn drag_target_at(&mut self, point: UiPoint) -> Option<HostDragTargetGroup> {
        self.drag_route_at(point).and_then(|route| match route {
            HostShellPointerRoute::DragTarget(group) => Some(group),
            HostShellPointerRoute::DocumentEdge(_)
            | HostShellPointerRoute::FloatingWindow(_)
            | HostShellPointerRoute::FloatingWindowEdge { .. } => {
                Some(HostDragTargetGroup::Document)
            }
            HostShellPointerRoute::Resize(_) => None,
        })
    }

    pub(crate) fn drag_route_at(&mut self, point: UiPoint) -> Option<HostShellPointerRoute> {
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
    pub(crate) fn resize_target_at(&mut self, point: UiPoint) -> Option<HostResizeTargetGroup> {
        self.dispatch_resize_event(UiPointerEvent::new(UiPointerEventKind::Move, point))
    }

    pub(crate) fn begin_resize(&mut self, point: UiPoint) -> Option<HostShellPointerRoute> {
        self.dispatch_resize_event(
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .map(HostShellPointerRoute::Resize)
    }

    pub(crate) fn update_resize(&mut self, point: UiPoint) -> Option<HostResizeTargetGroup> {
        self.dispatch_resize_event(UiPointerEvent::new(UiPointerEventKind::Move, point))
    }

    pub(crate) fn finish_resize(&mut self, point: UiPoint) -> Option<HostResizeTargetGroup> {
        self.dispatch_resize_event(
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
    }

    fn dispatch_resize_event(&mut self, event: UiPointerEvent) -> Option<HostResizeTargetGroup> {
        let dispatch = self
            .resize_surface
            .dispatch_pointer_event(&self.resize_dispatcher, event)
            .ok()?;
        resize_group_from_dispatch(&dispatch)
    }
}

use std::sync::Arc;

use arc_swap::ArcSwap;
use zircon_runtime::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiInputDiagnosticsMode, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiPointerEvent, UiPointerId,
        UiPointerInputEvent,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
};

#[cfg(test)]
use crate::ui::host::NativeWindowHostState;
#[cfg(test)]
use crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::drawer_resize::HostResizeTargetGroup;
#[cfg(test)]
use crate::ui::retained_host::floating_window_projection::build_floating_window_projection_bundle_from_windows;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::route_intent::EditorRouteIntentMap;
use crate::ui::retained_host::tab_drag::HostDragTargetGroup;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use crate::ui::workbench::autolayout::ShellSizePx;
#[cfg(test)]
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::FloatingWindowModel;

use super::drag_frames::DragHitGeometry;
use super::drag_surface::{build_drag_surface, patch_drag_surface};
use super::resize_surface::{build_resize_surface, update_resize_surface};
use super::route::HostShellPointerRoute;

const SHELL_POINTER_ID: UiPointerId = UiPointerId::new(1);

pub(crate) struct HostShellPointerBridge {
    drag_surface: UiSurface,
    drag_dispatcher: UiPointerDispatcher,
    drag_route_intents: EditorRouteIntentMap,
    drag_hit_geometry: Arc<ArcSwap<DragHitGeometry>>,
    drag_window_ids: Vec<MainPageId>,
    drag_topology_initialized: bool,
    #[cfg(test)]
    drag_surface_authority_generation: u64,
    resize_surface: UiSurface,
    resize_dispatcher: UiPointerDispatcher,
    resize_route_intents: EditorRouteIntentMap,
    navigation_dispatcher: UiNavigationDispatcher,
    input_sequence: u64,
}

impl Default for HostShellPointerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl HostShellPointerBridge {
    pub(crate) fn new() -> Self {
        let (drag_surface, drag_dispatcher, drag_route_intents, drag_hit_geometry) =
            build_drag_surface(
                ShellSizePx::new(1.0, 1.0),
                false,
                &[],
                BuiltinWorkbenchWindowLayoutFrames::default(),
                None,
            );
        let (resize_surface, resize_dispatcher, resize_route_intents) = build_resize_surface();
        Self {
            drag_surface,
            drag_dispatcher,
            drag_route_intents,
            drag_hit_geometry,
            drag_window_ids: Vec::new(),
            drag_topology_initialized: false,
            #[cfg(test)]
            drag_surface_authority_generation: 1,
            resize_surface,
            resize_dispatcher,
            resize_route_intents,
            navigation_dispatcher: UiNavigationDispatcher::default(),
            input_sequence: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn update_layout_with_floating_windows(
        &mut self,
        root_size: ShellSizePx,
        drawers_visible: bool,
        floating_windows: &[FloatingWindowModel],
    ) {
        let floating_window_projection_bundle =
            build_floating_window_projection_bundle_from_windows(
                floating_windows,
                None,
                &WorkbenchChromeMetrics::default(),
                &[],
            );
        self.update_layout_with_root_shell_frames(
            root_size,
            drawers_visible,
            floating_windows,
            None,
            Some(&floating_window_projection_bundle),
        );
    }

    #[cfg(test)]
    pub(crate) fn update_layout_with_native_window_hosts(
        &mut self,
        root_size: ShellSizePx,
        drawers_visible: bool,
        floating_windows: &[FloatingWindowModel],
        shared_root_frames: Option<&BuiltinHostRootShellFrames>,
        native_window_hosts: &[NativeWindowHostState],
    ) {
        let floating_window_projection_bundle =
            build_floating_window_projection_bundle_from_windows(
                floating_windows,
                None,
                &WorkbenchChromeMetrics::default(),
                native_window_hosts,
            );
        self.update_layout_with_root_shell_frames(
            root_size,
            drawers_visible,
            floating_windows,
            shared_root_frames,
            Some(&floating_window_projection_bundle),
        );
    }

    #[cfg(test)]
    pub(crate) fn update_layout_with_root_shell_frames(
        &mut self,
        root_size: ShellSizePx,
        drawers_visible: bool,
        floating_windows: &[FloatingWindowModel],
        shared_root_frames: Option<&BuiltinHostRootShellFrames>,
        floating_window_projection_bundle: Option<&FloatingWindowProjectionBundle>,
    ) {
        self.update_layout_with_workbench_layout_frames(
            root_size,
            drawers_visible,
            floating_windows,
            test_workbench_layout_frames_from_root_frames(shared_root_frames),
            floating_window_projection_bundle,
        );
    }

    pub(crate) fn update_layout_with_workbench_layout_frames(
        &mut self,
        root_size: ShellSizePx,
        drawers_visible: bool,
        floating_windows: &[FloatingWindowModel],
        componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
        floating_window_projection_bundle: Option<&FloatingWindowProjectionBundle>,
    ) {
        let stable_topology = self.drag_window_ids.len() == floating_windows.len()
            && self
                .drag_window_ids
                .iter()
                .zip(floating_windows)
                .all(|(known, current)| known == &current.window_id);
        let patched = stable_topology
            && patch_drag_surface(
                &mut self.drag_surface,
                &self.drag_hit_geometry,
                root_size,
                drawers_visible,
                floating_windows,
                componentized_workbench_layout_frames,
                floating_window_projection_bundle,
            )
            .is_some();
        if !patched {
            let record_rebuild = self.drag_topology_initialized;
            let (drag_surface, drag_dispatcher, drag_route_intents, drag_hit_geometry) =
                build_drag_surface(
                    root_size,
                    drawers_visible,
                    floating_windows,
                    componentized_workbench_layout_frames,
                    floating_window_projection_bundle,
                );
            self.drag_surface = drag_surface;
            self.drag_dispatcher = drag_dispatcher;
            self.drag_route_intents = drag_route_intents;
            self.drag_hit_geometry = drag_hit_geometry;
            self.drag_window_ids = floating_windows
                .iter()
                .map(|window| window.window_id.clone())
                .collect();
            if record_rebuild {
                record_current_ui_perf_counter(UiPerfCounter::ShellDragAuthorityRebuildCount, 1.0);
                record_current_ui_perf_counter(
                    UiPerfCounter::ShellDragNodeInsertCount,
                    (9usize.saturating_add(floating_windows.len().saturating_mul(5))) as f64,
                );
                record_current_ui_perf_counter(UiPerfCounter::ShellDragDispatcherRebuildCount, 1.0);
                record_current_ui_perf_counter(UiPerfCounter::ShellDragRouteMapRebuildCount, 1.0);
            }
            #[cfg(test)]
            {
                self.drag_surface_authority_generation =
                    self.drag_surface_authority_generation.saturating_add(1);
            }
        }
        self.drag_topology_initialized = true;
        update_resize_surface(
            &mut self.resize_surface,
            root_size,
            componentized_workbench_layout_frames,
        );
    }

    #[cfg(test)]
    pub(crate) const fn drag_surface_authority_generation_for_test(&self) -> u64 {
        self.drag_surface_authority_generation
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
        let dispatch =
            self.dispatch_drag_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        shell_pointer_route_from_input_result(&self.drag_route_intents, &dispatch)
    }

    #[cfg(test)]
    pub(crate) fn resize_target_at(&mut self, point: UiPoint) -> Option<HostResizeTargetGroup> {
        self.dispatch_resize_event(UiPointerEvent::new(UiPointerEventKind::Move, point))
            .and_then(resize_group_from_shell_route)
    }

    pub(crate) fn begin_resize(&mut self, point: UiPoint) -> Option<HostShellPointerRoute> {
        self.dispatch_resize_event(
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
    }

    pub(crate) fn update_resize(&mut self, point: UiPoint) -> Option<HostResizeTargetGroup> {
        self.dispatch_resize_event(UiPointerEvent::new(UiPointerEventKind::Move, point))
            .and_then(resize_group_from_shell_route)
    }

    pub(crate) fn cancel_resize(&mut self) {
        let _ = self.resize_surface.release_pointer_capture();
    }

    pub(crate) fn finish_resize(&mut self, point: UiPoint) -> Option<HostResizeTargetGroup> {
        self.dispatch_resize_event(
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .and_then(resize_group_from_shell_route)
    }

    fn dispatch_drag_event(&mut self, event: UiPointerEvent) -> Option<UiInputDispatchResult> {
        let event = self.pointer_input_event(event);
        self.drag_surface
            .dispatch_input_event_with_diagnostics_mode(
                &self.drag_dispatcher,
                &self.navigation_dispatcher,
                event,
                UiInputDiagnosticsMode::Summary,
            )
            .ok()
    }

    fn dispatch_resize_event(&mut self, event: UiPointerEvent) -> Option<HostShellPointerRoute> {
        let event = self.pointer_input_event(event);
        let dispatch = self
            .resize_surface
            .dispatch_input_event_with_diagnostics_mode(
                &self.resize_dispatcher,
                &self.navigation_dispatcher,
                event,
                UiInputDiagnosticsMode::Summary,
            )
            .ok()?;
        shell_pointer_route_from_input_result(&self.resize_route_intents, &dispatch)
    }

    fn pointer_input_event(&mut self, event: UiPointerEvent) -> UiInputEvent {
        self.input_sequence += 1;
        let mut metadata = UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(self.input_sequence),
            UiInputSequence::new(self.input_sequence),
        );
        metadata.pointer_id = Some(SHELL_POINTER_ID);
        UiInputEvent::Pointer(UiPointerInputEvent {
            metadata,
            event,
            precise_scroll: None,
        })
    }
}

fn shell_pointer_route_from_input_result(
    intents: &EditorRouteIntentMap,
    result: &UiInputDispatchResult,
) -> Option<HostShellPointerRoute> {
    shell_pointer_reply_effect_target(result)
        .or(result.reply.handler)
        .or_else(|| {
            result
                .pointer_routing
                .as_ref()
                .and_then(|routing| routing.route_target)
        })
        .and_then(|node_id| intents.shell_pointer_route_for_node(node_id))
}

fn shell_pointer_reply_effect_target(result: &UiInputDispatchResult) -> Option<UiNodeId> {
    result.reply.effects.iter().find_map(|effect| match effect {
        UiDispatchEffect::CapturePointer { target, .. }
        | UiDispatchEffect::ReleasePointerCapture { target, .. } => Some(*target),
        _ => None,
    })
}

fn resize_group_from_shell_route(route: HostShellPointerRoute) -> Option<HostResizeTargetGroup> {
    match route {
        HostShellPointerRoute::Resize(group) => Some(group),
        HostShellPointerRoute::DragTarget(_)
        | HostShellPointerRoute::DocumentEdge(_)
        | HostShellPointerRoute::FloatingWindow(_)
        | HostShellPointerRoute::FloatingWindowEdge { .. } => None,
    }
}

#[cfg(test)]
fn test_workbench_layout_frames_from_root_frames(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> BuiltinWorkbenchWindowLayoutFrames {
    BuiltinWorkbenchWindowLayoutFrames {
        center_band_frame: shared_root_frames.and_then(|frames| frames.host_body_frame),
        document_region_frame: shared_root_frames.and_then(|frames| frames.document_host_frame),
        status_bar_frame: shared_root_frames.and_then(|frames| frames.status_bar_frame),
        ..BuiltinWorkbenchWindowLayoutFrames::default()
    }
}

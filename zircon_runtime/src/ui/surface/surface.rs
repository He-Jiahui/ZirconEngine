use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
use crate::ui::layout::compute_layout_tree;
use crate::ui::tree::{
    UiHitTestIndex, UiHitTestResult, UiRuntimeTreeAccessExt, UiRuntimeTreeFocusExt,
    UiRuntimeTreeInteractionExt, UiRuntimeTreeRoutingExt, UiRuntimeTreeScrollExt,
};
use zircon_runtime_interface::ui::dispatch::{
    UiPointerComponentEvent, UiPointerComponentEventReason,
};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTree, UiTreeError};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::{UiNavigationDispatchResult, UiPointerDispatchResult, UiPointerEvent},
    event_ui::{UiNodeId, UiReflectorSnapshot, UiTreeId},
    layout::{UiPoint, UiSize},
    surface::{
        UiArrangedTree, UiFocusState, UiHitTestDebugDump, UiHitTestQuery, UiNavigationEventKind,
        UiNavigationRoute, UiNavigationState, UiPointerActivationPhase, UiPointerButton,
        UiPointerEventKind, UiPointerRoute, UiRenderExtract, UiRenderList, UiSurfaceDebugOptions,
        UiSurfaceDebugSnapshot, UiSurfaceFrame, UiSurfaceRebuildDebugStats,
    },
};

use super::{
    build_arranged_tree, debug_hit_test_surface_frame, debug_surface_frame,
    debug_surface_frame_with_options, extract_ui_render_tree_from_arranged,
    property_mutation::{
        mutate_tree_property, UiPropertyMutationReport, UiPropertyMutationRequest,
    },
    reflector_snapshot,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfaceRebuildReport {
    pub dirty_flags: UiDirtyFlags,
    pub dirty_node_count: usize,
    pub layout_recomputed: bool,
    pub arranged_rebuilt: bool,
    pub hit_grid_rebuilt: bool,
    pub render_rebuilt: bool,
    pub arranged_node_count: usize,
    pub render_command_count: usize,
    pub hit_grid_entry_count: usize,
    pub hit_grid_cell_count: usize,
    pub layout_elapsed_micros: u64,
    pub arranged_elapsed_micros: u64,
    pub hit_grid_elapsed_micros: u64,
    pub render_elapsed_micros: u64,
}

impl UiSurfaceRebuildReport {
    pub fn debug_stats(self) -> UiSurfaceRebuildDebugStats {
        UiSurfaceRebuildDebugStats {
            dirty_flags: self.dirty_flags,
            dirty_node_count: self.dirty_node_count,
            layout_recomputed: self.layout_recomputed,
            arranged_rebuilt: self.arranged_rebuilt,
            hit_grid_rebuilt: self.hit_grid_rebuilt,
            render_rebuilt: self.render_rebuilt,
            arranged_node_count: self.arranged_node_count,
            render_command_count: self.render_command_count,
            hit_grid_entry_count: self.hit_grid_entry_count,
            hit_grid_cell_count: self.hit_grid_cell_count,
            layout_elapsed_micros: self.layout_elapsed_micros,
            arranged_elapsed_micros: self.arranged_elapsed_micros,
            hit_grid_elapsed_micros: self.hit_grid_elapsed_micros,
            render_elapsed_micros: self.render_elapsed_micros,
        }
    }

    fn with_counts(mut self, counts: UiSurfaceRebuildReport) -> Self {
        self.arranged_node_count = counts.arranged_node_count;
        self.render_command_count = counts.render_command_count;
        self.hit_grid_entry_count = counts.hit_grid_entry_count;
        self.hit_grid_cell_count = counts.hit_grid_cell_count;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurface {
    pub tree: UiTree,
    pub arranged_tree: UiArrangedTree,
    pub hit_test: UiHitTestIndex,
    pub focus: UiFocusState,
    pub navigation: UiNavigationState,
    pub render_extract: UiRenderExtract,
    pub last_rebuild_report: UiSurfaceRebuildReport,
}

impl UiSurface {
    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree: UiTree::new(tree_id.clone()),
            arranged_tree: UiArrangedTree {
                tree_id: tree_id.clone(),
                ..Default::default()
            },
            hit_test: UiHitTestIndex::default(),
            focus: UiFocusState::default(),
            navigation: UiNavigationState::default(),
            render_extract: UiRenderExtract {
                tree_id,
                list: UiRenderList::default(),
            },
            last_rebuild_report: UiSurfaceRebuildReport::default(),
        }
    }

    fn rebuild_counts(&self) -> UiSurfaceRebuildReport {
        UiSurfaceRebuildReport {
            arranged_node_count: self.arranged_tree.nodes.len(),
            render_command_count: self.render_extract.list.commands.len(),
            hit_grid_entry_count: self.hit_test.grid.entries.len(),
            hit_grid_cell_count: self.hit_test.grid.cells.len(),
            ..UiSurfaceRebuildReport::default()
        }
    }

    pub fn rebuild(&mut self) {
        let dirty_flags = self.dirty_flags();
        let dirty_node_count = dirty_node_count(&self.tree);
        let arranged_start = Instant::now();
        self.arranged_tree = build_arranged_tree(&self.tree);
        let arranged_elapsed_micros = elapsed_micros(arranged_start);
        let hit_start = Instant::now();
        self.hit_test.rebuild_arranged(&self.arranged_tree);
        let hit_grid_elapsed_micros = elapsed_micros(hit_start);
        let render_start = Instant::now();
        self.render_extract = extract_ui_render_tree_from_arranged(&self.tree, &self.arranged_tree);
        let render_elapsed_micros = elapsed_micros(render_start);
        self.last_rebuild_report = UiSurfaceRebuildReport {
            dirty_flags,
            dirty_node_count,
            arranged_rebuilt: true,
            hit_grid_rebuilt: true,
            render_rebuilt: true,
            arranged_elapsed_micros,
            hit_grid_elapsed_micros,
            render_elapsed_micros,
            ..self.rebuild_counts()
        };
    }

    pub fn dirty_flags(&self) -> UiDirtyFlags {
        self.tree
            .nodes
            .values()
            .fold(UiDirtyFlags::default(), merge_dirty_flags)
    }

    pub fn clear_dirty_flags(&mut self) {
        for node in self.tree.nodes.values_mut() {
            node.dirty = UiDirtyFlags::default();
            node.state_flags.dirty = false;
        }
    }

    pub fn rebuild_dirty(
        &mut self,
        root_size: UiSize,
    ) -> Result<UiSurfaceRebuildReport, UiTreeError> {
        let dirty = self.dirty_flags();
        let dirty_node_count = dirty_node_count(&self.tree);
        if !dirty.any() {
            self.last_rebuild_report = UiSurfaceRebuildReport::default().with_counts(self.rebuild_counts());
            return Ok(self.last_rebuild_report);
        }

        if dirty.layout || dirty.style || dirty.text || dirty.visible_range {
            let layout_start = Instant::now();
            compute_layout_tree(&mut self.tree, root_size)?;
            let layout_elapsed_micros = elapsed_micros(layout_start);
            let arranged_start = Instant::now();
            self.arranged_tree = build_arranged_tree(&self.tree);
            let arranged_elapsed_micros = elapsed_micros(arranged_start);
            let hit_start = Instant::now();
            self.hit_test.rebuild_arranged(&self.arranged_tree);
            let hit_grid_elapsed_micros = elapsed_micros(hit_start);
            let render_start = Instant::now();
            self.render_extract =
                extract_ui_render_tree_from_arranged(&self.tree, &self.arranged_tree);
            let render_elapsed_micros = elapsed_micros(render_start);
            let report = UiSurfaceRebuildReport {
                dirty_flags: dirty,
                dirty_node_count,
                layout_recomputed: true,
                arranged_rebuilt: true,
                hit_grid_rebuilt: true,
                render_rebuilt: true,
                layout_elapsed_micros,
                arranged_elapsed_micros,
                hit_grid_elapsed_micros,
                render_elapsed_micros,
                ..self.rebuild_counts()
            };
            self.last_rebuild_report = report;
            self.clear_dirty_flags();
            return Ok(report);
        }

        let mut report = UiSurfaceRebuildReport {
            dirty_flags: dirty,
            dirty_node_count,
            ..UiSurfaceRebuildReport::default()
        };
        if dirty.hit_test || dirty.input {
            let arranged_start = Instant::now();
            self.arranged_tree = build_arranged_tree(&self.tree);
            report.arranged_elapsed_micros = elapsed_micros(arranged_start);
            let hit_start = Instant::now();
            self.hit_test.rebuild_arranged(&self.arranged_tree);
            report.hit_grid_elapsed_micros = elapsed_micros(hit_start);
            report.arranged_rebuilt = true;
            report.hit_grid_rebuilt = true;
        }
        if dirty.render {
            let render_start = Instant::now();
            self.render_extract =
                extract_ui_render_tree_from_arranged(&self.tree, &self.arranged_tree);
            report.render_elapsed_micros = elapsed_micros(render_start);
            report.render_rebuilt = true;
        }
        report = UiSurfaceRebuildReport {
            ..report.with_counts(self.rebuild_counts())
        };
        self.last_rebuild_report = report;
        self.clear_dirty_flags();
        Ok(report)
    }

    pub fn compute_layout(&mut self, root_size: UiSize) -> Result<(), UiTreeError> {
        compute_layout_tree(&mut self.tree, root_size)?;
        self.rebuild();
        self.clear_dirty_flags();
        Ok(())
    }

    pub fn hit_test(&self, point: UiPoint) -> UiHitTestResult {
        self.hit_test.hit_test_arranged(&self.arranged_tree, point)
    }

    pub fn hit_test_with_query(&self, query: UiHitTestQuery) -> UiHitTestResult {
        self.hit_test
            .hit_test_arranged_with_query(&self.arranged_tree, query)
    }

    pub fn surface_frame(&self) -> UiSurfaceFrame {
        UiSurfaceFrame {
            tree_id: self.tree.tree_id.clone(),
            arranged_tree: self.arranged_tree.clone(),
            render_extract: self.render_extract.clone(),
            hit_grid: self.hit_test.grid.clone(),
            focus_state: self.focus.clone(),
            last_rebuild: self.last_rebuild_report.debug_stats(),
        }
    }

    pub fn debug_hit_test(&self, point: UiPoint) -> UiHitTestDebugDump {
        debug_hit_test_surface_frame(&self.surface_frame(), point)
    }

    pub fn debug_snapshot(&self) -> UiSurfaceDebugSnapshot {
        debug_surface_frame(&self.surface_frame())
    }

    pub fn debug_snapshot_with_options(
        &self,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        debug_surface_frame_with_options(&self.surface_frame(), options)
    }

    pub fn mutate_property(
        &mut self,
        request: UiPropertyMutationRequest,
    ) -> Result<UiPropertyMutationReport, UiTreeError> {
        mutate_tree_property(&mut self.tree, request)
    }

    pub fn reflector_snapshot(&self, query: Option<UiHitTestQuery>) -> UiReflectorSnapshot {
        reflector_snapshot(self, query)
    }

    pub fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
        self.tree.bubble_route(node_id)
    }

    pub fn focused_route(&self) -> Vec<UiNodeId> {
        self.focus
            .focused
            .and_then(|node_id| self.tree.bubble_route(node_id).ok())
            .unwrap_or_default()
    }

    pub fn focus_node(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.is_focus_candidate() {
            return Err(UiTreeError::MissingNode(node_id));
        }
        self.focus.focused = Some(node_id);
        self.navigation.navigation_root = Some(node_id);
        self.navigation.focus_visible = true;
        Ok(())
    }

    pub fn clear_focus(&mut self) {
        self.focus.focused = None;
        self.navigation.navigation_root = None;
        self.navigation.focus_visible = false;
    }

    pub fn capture_pointer(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        self.tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        self.focus.captured = Some(node_id);
        Ok(())
    }

    pub fn release_pointer_capture(&mut self) -> Option<UiNodeId> {
        self.focus.captured.take()
    }

    pub fn route_pointer_event(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(kind, UiHitTestQuery::new(point), None, 0.0)
    }

    pub fn route_pointer_event_with_query(
        &mut self,
        kind: UiPointerEventKind,
        query: UiHitTestQuery,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(kind, query, None, 0.0)
    }

    pub fn route_pointer_event_with_button(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
        button: UiPointerButton,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(kind, UiHitTestQuery::new(point), Some(button), 0.0)
    }

    pub fn route_pointer_event_with_query_and_button(
        &mut self,
        kind: UiPointerEventKind,
        query: UiHitTestQuery,
        button: UiPointerButton,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(kind, query, Some(button), 0.0)
    }

    pub fn dispatch_pointer_event(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let point = event.point;
        self.dispatch_pointer_event_with_query(dispatcher, event, UiHitTestQuery::new(point))
    }

    pub fn dispatch_pointer_event_with_query(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
        query: UiHitTestQuery,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let route = self.route_pointer_event_with_details(
            event.kind,
            query,
            event.button,
            event.scroll_delta,
        )?;
        let focus_before_dispatch = self.focus.focused;
        let capture_before_dispatch = route.captured;
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if let Some(node_id) = result.captured_by {
            self.focus.captured = Some(node_id);
        } else if matches!(event.kind, UiPointerEventKind::Scroll)
            && result.handled_by.is_none()
            && result.blocked_by.is_none()
        {
            let candidates = if !route.stacked.is_empty() {
                route.stacked.as_slice()
            } else {
                route.root_targets.as_slice()
            };
            if let Some(node_id) = self.tree.first_scrollable_in_candidates(candidates)? {
                let _ = self.tree.scroll_by(node_id, event.scroll_delta)?;
                result.handled_by = Some(node_id);
            }
        }
        result.diagnostics.focus_changed = focus_before_dispatch != self.focus.focused;
        result.diagnostics.capture_released = matches!(event.kind, UiPointerEventKind::Up)
            && capture_before_dispatch.is_some()
            && self.focus.captured.is_none();
        result.component_events = self.pointer_component_events(&route)?;
        Ok(result)
    }

    fn pointer_component_events(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Vec<UiPointerComponentEvent>, UiTreeError> {
        let mut events = Vec::new();
        for node_id in &route.entered {
            self.push_pointer_component_events(
                &mut events,
                *node_id,
                UiEventKind::Hover,
                UiComponentEvent::Hover { hovered: true },
                UiPointerComponentEventReason::HoverEnter,
            )?;
        }
        for node_id in &route.left {
            self.push_pointer_component_events(
                &mut events,
                *node_id,
                UiEventKind::Hover,
                UiComponentEvent::Hover { hovered: false },
                UiPointerComponentEventReason::HoverLeave,
            )?;
        }

        if route.activation_phase == UiPointerActivationPhase::PrimaryPress {
            if let Some(node_id) = route.target {
                self.push_pointer_component_events(
                    &mut events,
                    node_id,
                    UiEventKind::Press,
                    UiComponentEvent::Press { pressed: true },
                    UiPointerComponentEventReason::PressBegin,
                )?;
            }
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryRelease {
            if let Some(node_id) = route.pressed {
                self.push_pointer_component_events(
                    &mut events,
                    node_id,
                    UiEventKind::Release,
                    UiComponentEvent::Press { pressed: false },
                    UiPointerComponentEventReason::PressEnd,
                )?;
            }
            if let Some(node_id) = route.click_target {
                self.push_pointer_component_events(
                    &mut events,
                    node_id,
                    UiEventKind::Click,
                    UiComponentEvent::Commit {
                        property: "activated".to_string(),
                        value: zircon_runtime_interface::ui::component::UiValue::Bool(true),
                    },
                    UiPointerComponentEventReason::DefaultClick,
                )?;
            }
        }

        Ok(events)
    }

    fn push_pointer_component_events(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        node_id: UiNodeId,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        reason: UiPointerComponentEventReason,
    ) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(());
        };
        let control_id = metadata
            .control_id
            .as_deref()
            .unwrap_or(node.node_path.0.as_str());
        for binding in metadata
            .bindings
            .iter()
            .filter(|binding| binding.event == event_kind)
        {
            events.push(UiPointerComponentEvent::new(
                &self.tree.tree_id,
                node_id,
                control_id,
                binding.id.as_str(),
                event_kind,
                event.clone(),
                reason,
            ));
        }
        Ok(())
    }

    fn route_pointer_event_with_details(
        &mut self,
        kind: UiPointerEventKind,
        query: UiHitTestQuery,
        button: Option<UiPointerButton>,
        scroll_delta: f32,
    ) -> Result<UiPointerRoute, UiTreeError> {
        let point = query.hit_point();
        let hit = self.hit_test_with_query(query);
        let previous_hovered = self.focus.hovered.clone();
        let captured = self.focus.captured;
        let previous_pressed = self.focus.pressed;
        let target = captured.or(hit.top_hit);
        let bubbled = match target {
            Some(node_id) => self.tree.bubble_route(node_id)?,
            None => Vec::new(),
        };

        self.focus.hovered = hit.stacked.clone();
        if matches!(kind, UiPointerEventKind::Down) {
            self.focus.pressed = target;
            if let Some(focus_target) = self.tree.first_focusable_in_route(&bubbled)? {
                self.focus_node(focus_target)?;
            }
        }
        let click_target = if matches!(kind, UiPointerEventKind::Up)
            && button == Some(UiPointerButton::Primary)
            && previous_pressed.is_some_and(|node_id| hit.stacked.contains(&node_id))
        {
            previous_pressed
        } else {
            None
        };
        if matches!(kind, UiPointerEventKind::Up) {
            self.focus.pressed = None;
            self.focus.captured = None;
        }
        let pressed = if matches!(kind, UiPointerEventKind::Down) {
            self.focus.pressed
        } else {
            previous_pressed
        };

        Ok(UiPointerRoute {
            kind,
            button,
            activation_phase: activation_phase(kind, button),
            point,
            scroll_delta,
            target,
            hit_path: hit.path.clone(),
            bubbled,
            stacked: hit.stacked.clone(),
            entered: diff_nodes(&hit.stacked, &previous_hovered),
            left: diff_nodes(&previous_hovered, &hit.stacked),
            captured,
            pressed,
            click_target,
            release_inside_pressed: click_target.is_some(),
            focused: self.focus.focused,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }

    pub fn route_navigation_event(
        &self,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationRoute, UiTreeError> {
        let target = self.focus.focused.or(self.navigation.navigation_root);
        let bubbled = match target {
            Some(node_id) => self.tree.bubble_route(node_id)?,
            None => Vec::new(),
        };
        Ok(UiNavigationRoute {
            kind,
            target,
            bubbled,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }

    pub fn dispatch_navigation_event(
        &mut self,
        dispatcher: &UiNavigationDispatcher,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        let route = self.route_navigation_event(kind)?;
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if result.focus_changed_to.is_none() {
            if let Some(node_id) = self.tree.next_focusable_target(route.target, route.kind)? {
                result.handled_by = Some(route.target.unwrap_or(node_id));
                result.focus_changed_to = Some(node_id);
            }
        }
        if let Some(node_id) = result.focus_changed_to {
            self.focus_node(node_id)?;
        }
        Ok(result)
    }
}

fn diff_nodes(current: &[UiNodeId], previous: &[UiNodeId]) -> Vec<UiNodeId> {
    current
        .iter()
        .filter(|node_id| !previous.contains(node_id))
        .copied()
        .collect()
}

fn merge_dirty_flags(
    mut dirty: UiDirtyFlags,
    node: &zircon_runtime_interface::ui::tree::UiTreeNode,
) -> UiDirtyFlags {
    dirty.layout |= node.dirty.layout;
    dirty.hit_test |= node.dirty.hit_test || node.state_flags.dirty;
    dirty.render |= node.dirty.render || node.state_flags.dirty;
    dirty.style |= node.dirty.style;
    dirty.text |= node.dirty.text;
    dirty.input |= node.dirty.input || node.state_flags.dirty;
    dirty.visible_range |= node.dirty.visible_range;
    dirty
}

fn dirty_node_count(tree: &UiTree) -> usize {
    tree.nodes
        .values()
        .filter(|node| node.dirty.any() || node.state_flags.dirty)
        .count()
}

fn elapsed_micros(start: Instant) -> u64 {
    start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

fn activation_phase(
    kind: UiPointerEventKind,
    button: Option<UiPointerButton>,
) -> UiPointerActivationPhase {
    match (kind, button) {
        (UiPointerEventKind::Down, Some(UiPointerButton::Primary)) => {
            UiPointerActivationPhase::PrimaryPress
        }
        (UiPointerEventKind::Up, Some(UiPointerButton::Primary)) => {
            UiPointerActivationPhase::PrimaryRelease
        }
        (UiPointerEventKind::Down, Some(UiPointerButton::Secondary)) => {
            UiPointerActivationPhase::SecondaryPress
        }
        (UiPointerEventKind::Up, Some(UiPointerButton::Secondary)) => {
            UiPointerActivationPhase::SecondaryRelease
        }
        (UiPointerEventKind::Down, Some(UiPointerButton::Middle)) => {
            UiPointerActivationPhase::MiddlePress
        }
        (UiPointerEventKind::Up, Some(UiPointerButton::Middle)) => {
            UiPointerActivationPhase::MiddleRelease
        }
        (UiPointerEventKind::Move, _) => UiPointerActivationPhase::Hover,
        (UiPointerEventKind::Scroll, _) => UiPointerActivationPhase::Scroll,
        _ => UiPointerActivationPhase::None,
    }
}

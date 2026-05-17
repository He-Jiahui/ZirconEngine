use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
use crate::ui::tree::{
    UiHitTestIndex, UiHitTestResult, UiRuntimeTreeAccessExt, UiRuntimeTreeFocusExt,
    UiRuntimeTreeInteractionExt, UiRuntimeTreeRoutingExt, UiRuntimeTreeScrollExt,
};
use crate::ui::v2::UiV2RuntimeStyleIndex;
use zircon_runtime_interface::ui::accessibility::UiAccessibilityTreeSnapshot;
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchReply, UiDispatchReplyStep, UiInputDispatchResult, UiInputEvent,
    UiPointerComponentEvent, UiPointerComponentEventReason, UiPointerDispatchEffect,
};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTree, UiTreeError};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{UiNavigationDispatchResult, UiPointerDispatchResult, UiPointerEvent},
    event_ui::{UiNodeId, UiReflectorSnapshot, UiTreeId},
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
    layout::UiPoint,
    surface::{
        UiArrangedTree, UiFocusState, UiHitTestDebugDump, UiHitTestQuery, UiNavigationEventKind,
        UiNavigationRoute, UiNavigationState, UiPointerActivationPhase, UiPointerButton,
        UiPointerEventKind, UiPointerRoute, UiRenderExtract, UiRenderList, UiSurfaceDebugOptions,
        UiSurfaceDebugSnapshot, UiSurfaceFrame,
    },
};

use super::{
    component_state::UiSurfaceComponentStateStore,
    debug_hit_test_surface_frame, debug_surface_frame, debug_surface_frame_for_pick,
    debug_surface_frame_for_selection, debug_surface_frame_with_options,
    input::{
        apply_dispatch_reply, apply_dispatch_reply_steps, dispatch_input_event, UiSurfaceInputState,
    },
    node_pool::{UiSurfaceNodePool, UiSurfaceNodePoolReport},
    property_mutation::{
        mutate_tree_property, UiPropertyMutationReport, UiPropertyMutationRequest,
        UiPropertyMutationStatus,
    },
    reflector_snapshot,
    render::UiSurfaceRenderCache,
};

mod default_interactions;
mod interaction_state;
mod rebuild;

pub use rebuild::UiSurfaceRebuildReport;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurface {
    pub tree: UiTree,
    pub arranged_tree: UiArrangedTree,
    pub hit_test: UiHitTestIndex,
    pub focus: UiFocusState,
    #[serde(default)]
    pub input: UiSurfaceInputState,
    #[serde(default)]
    pub component_states: UiSurfaceComponentStateStore,
    #[serde(default, skip)]
    pub(crate) runtime_style: UiV2RuntimeStyleIndex,
    pub navigation: UiNavigationState,
    pub render_extract: UiRenderExtract,
    #[serde(default)]
    pub render_cache: UiSurfaceRenderCache,
    #[serde(default)]
    pub node_pool: UiSurfaceNodePool,
    pub last_rebuild_report: UiSurfaceRebuildReport,
    #[serde(default)]
    pub(super) pending_pool_report: UiSurfaceNodePoolReport,
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
            input: UiSurfaceInputState::default(),
            component_states: UiSurfaceComponentStateStore::default(),
            runtime_style: UiV2RuntimeStyleIndex::default(),
            navigation: UiNavigationState::default(),
            render_extract: UiRenderExtract {
                tree_id,
                list: UiRenderList::default(),
            },
            render_cache: UiSurfaceRenderCache::default(),
            node_pool: UiSurfaceNodePool::default(),
            last_rebuild_report: UiSurfaceRebuildReport::default(),
            pending_pool_report: UiSurfaceNodePoolReport::default(),
        }
    }

    pub fn component_state(
        &self,
        node_id: UiNodeId,
    ) -> Option<&zircon_runtime_interface::ui::component::UiComponentState> {
        self.component_states.get(node_id)
    }

    pub(crate) fn set_runtime_style_index(&mut self, runtime_style: UiV2RuntimeStyleIndex) {
        self.runtime_style = runtime_style;
    }

    pub(crate) fn seed_component_states_from_tree_metadata(&mut self) {
        for (node_id, node) in &self.tree.nodes {
            if let Some(metadata) = node.template_metadata.as_ref() {
                if bool_attribute(&metadata.attributes, "hovered")
                    || bool_attribute(&metadata.attributes, "hover")
                {
                    let _ = self.component_states.set_hovered(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "focused")
                    || bool_attribute(&metadata.attributes, "focus")
                {
                    let _ = self.component_states.set_focused(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "pressed")
                    || bool_attribute(&metadata.attributes, "active")
                {
                    let _ = self.component_states.set_pressed(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "checked") {
                    let _ = self.component_states.set_checked(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "disabled") {
                    let _ = self.component_states.set_disabled(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "expanded") {
                    let _ = self.component_states.set_expanded(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "popup_open")
                    || bool_attribute(&metadata.attributes, "open")
                {
                    let _ = self.component_states.set_popup_open(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "selected") {
                    let _ = self.component_states.set_selected(*node_id, true);
                }
            }
            if node.state_flags.pressed {
                let _ = self.component_states.set_pressed(*node_id, true);
            }
            if node.state_flags.checked {
                let _ = self.component_states.set_checked(*node_id, true);
            }
            if !node.state_flags.enabled {
                let _ = self.component_states.set_disabled(*node_id, true);
            }
        }
    }

    pub(crate) fn apply_runtime_state_style_all(
        &mut self,
        mark_dirty: bool,
    ) -> Result<usize, UiTreeError> {
        if !self.runtime_style.has_runtime_rules() {
            return Ok(0);
        }
        let roots = self.tree.roots.clone();
        let mut changed = 0;
        for root in roots {
            changed += self.apply_runtime_state_style_subtree(root, mark_dirty)?;
        }
        Ok(changed)
    }

    pub(crate) fn apply_runtime_state_style_subtree(
        &mut self,
        root_id: UiNodeId,
        mark_dirty: bool,
    ) -> Result<usize, UiTreeError> {
        self.runtime_style.apply_to_tree_subtree(
            &mut self.tree,
            &self.component_states,
            root_id,
            mark_dirty,
        )
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

    pub fn accessibility_snapshot(&self) -> UiAccessibilityTreeSnapshot {
        crate::ui::accessibility::accessibility_snapshot(self)
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

    pub fn debug_snapshot_for_pick(
        &self,
        query: UiHitTestQuery,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        debug_surface_frame_for_pick(&self.surface_frame(), query, options)
    }

    pub fn debug_snapshot_for_selection(
        &self,
        selected_node: UiNodeId,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        debug_surface_frame_for_selection(&self.surface_frame(), selected_node, options)
    }

    pub fn debug_snapshot_json(
        &self,
        options: &UiSurfaceDebugOptions,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.debug_snapshot_with_options(options))
    }

    pub fn mutate_property(
        &mut self,
        request: UiPropertyMutationRequest,
    ) -> Result<UiPropertyMutationReport, UiTreeError> {
        let node_id = request.node_id;
        let property = request.property.clone();
        let value = request.value.clone();
        let mut report = mutate_tree_property(&mut self.tree, request)?;
        let component_state_changed = if matches!(report.status, UiPropertyMutationStatus::Accepted)
        {
            self.sync_component_state_from_property(node_id, &property, &value)?
        } else {
            false
        };
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            if component_state_changed {
                self.mark_component_state_render_dirty(node_id)?;
                report.invalidation.dirty.render = true;
            } else if property_may_affect_runtime_pseudo_state(&property) {
                let changed = self.apply_runtime_state_style_subtree(node_id, true)?;
                if changed > 0 {
                    report.invalidation.dirty.render = true;
                }
            }
        }
        if matches!(report.status, UiPropertyMutationStatus::Accepted)
            && matches!(
                property.as_str(),
                "enabled" | "visible" | "visibility" | "focusable"
            )
        {
            let reason = focus_reconcile_reason(&property, &self.tree, node_id);
            report.focus_change = self.reconcile_focus_after_tree_change(reason);
        }
        Ok(report)
    }

    fn sync_component_state_from_property(
        &mut self,
        node_id: UiNodeId,
        property: &str,
        value: &UiValue,
    ) -> Result<bool, UiTreeError> {
        let mut changed =
            self.component_states
                .set_value(node_id, property.to_string(), value.clone());
        let UiValue::Bool(value) = value else {
            return Ok(changed);
        };
        match property {
            "pressed" => {
                changed |= self.component_states.set_pressed(node_id, *value);
            }
            "checked" => {
                changed |= self.component_states.set_checked(node_id, *value);
            }
            "enabled" => {
                changed |= self.component_states.set_disabled(node_id, !*value);
            }
            "disabled" => {
                changed |= self.component_states.set_disabled(node_id, *value);
            }
            "expanded" => {
                changed |= self.component_states.set_expanded(node_id, *value);
            }
            "popup_open" | "open" => {
                changed |= self.component_states.set_popup_open(node_id, *value);
            }
            "selected" => {
                changed |= self.component_states.set_selected(node_id, *value);
            }
            _ => {}
        }
        Ok(changed)
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

    pub fn capture_pointer(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        if !super::input::is_valid_input_owner(self, node_id) {
            return Err(UiTreeError::MissingNode(node_id));
        }
        if let Some(previous) = self.focus.captured.filter(|owner| owner != &node_id) {
            self.input.clear_high_precision_for(previous);
        }
        self.input.clear_pointer_capture_for(node_id);
        self.focus.captured = Some(node_id);
        Ok(())
    }

    pub fn release_pointer_capture(&mut self) -> Option<UiNodeId> {
        let released = self.focus.captured.take();
        if let Some(owner) = released {
            self.input.clear_pointer_capture_for(owner);
        } else {
            self.input.clear_pointer_capture();
        }
        released
    }

    pub fn apply_dispatch_reply(
        &mut self,
        event: UiInputEvent,
        reply: UiDispatchReply,
    ) -> UiInputDispatchResult {
        apply_dispatch_reply(self, event, reply)
    }

    pub fn apply_dispatch_reply_steps(
        &mut self,
        event: UiInputEvent,
        steps: impl IntoIterator<Item = UiDispatchReplyStep>,
    ) -> UiInputDispatchResult {
        apply_dispatch_reply_steps(self, event, steps)
    }

    pub fn dispatch_input_event(
        &mut self,
        pointer_dispatcher: &UiPointerDispatcher,
        navigation_dispatcher: &UiNavigationDispatcher,
        event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        dispatch_input_event(self, pointer_dispatcher, navigation_dispatcher, event)
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
        let focus_before_dispatch = self.focus.focused;
        let capture_before_dispatch = self.focus.captured;
        let pressed_before_dispatch = self.focus.pressed;
        let _hover_before_dispatch = self.focus.hovered.clone();
        let route = self.route_pointer_event_with_details(
            event.kind,
            query,
            event.button,
            event.scroll_delta,
        )?;
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if let Some(node_id) = result.captured_by {
            if capture_before_dispatch != Some(node_id) {
                result.diagnostics.capture_started = true;
            }
            if let Some(previous) = capture_before_dispatch.filter(|owner| owner != &node_id) {
                self.input.clear_high_precision_for(previous);
            }
            self.focus.captured = Some(node_id);
            self.input.captured_pointer_id = None;
        }
        if let Some(node_id) = result.released_capture {
            if self.focus.captured == Some(node_id) || route.captured == Some(node_id) {
                self.focus.captured = None;
                self.input.clear_pointer_capture_for(node_id);
                result.diagnostics.capture_released = true;
            }
        }
        if let Some(node_id) = result.focus_changed_to {
            let focus_visible = result
                .invocations
                .iter()
                .rev()
                .find_map(|invocation| match invocation.effect {
                    UiPointerDispatchEffect::SetFocus { focus_visible }
                        if invocation.node_id == node_id =>
                    {
                        Some(focus_visible)
                    }
                    _ => None,
                })
                .unwrap_or(false);
            self.focus_node_with_reason(
                node_id,
                UiFocusChangeReason::Input,
                if focus_visible {
                    UiFocusVisible::visible(UiFocusVisibleReason::PointerInteraction)
                } else {
                    UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction)
                },
            )?;
        }
        if result.focus_cleared {
            self.clear_focus();
        }
        if matches!(event.kind, UiPointerEventKind::Scroll)
            && result.handled_by.is_none()
            && result.blocked_by.is_none()
        {
            let candidates = if !route.stacked.is_empty() {
                route.stacked.as_slice()
            } else {
                route.root_targets.as_slice()
            };
            for node_id in self.tree.scrollable_candidates(candidates)? {
                if self.tree.scroll_by(node_id, event.scroll_delta)? {
                    result.handled_by = Some(node_id);
                    result.diagnostics.scroll_defaulted = true;
                    break;
                }
            }
        }
        result.diagnostics.focus_changed = focus_before_dispatch != self.focus.focused;
        result.diagnostics.capture_released = result.diagnostics.capture_released
            || (matches!(event.kind, UiPointerEventKind::Up)
                && capture_before_dispatch.is_some()
                && self.focus.captured.is_none());
        if result.diagnostics.capture_released {
            if let Some(owner) = capture_before_dispatch
                .or(result.released_capture)
                .or(route.captured)
            {
                self.input.clear_pointer_capture_for(owner);
            } else {
                self.input.clear_pointer_capture();
            }
        }
        result.diagnostics.default_click_rejected = route.activation_phase
            == UiPointerActivationPhase::PrimaryRelease
            && route.pressed.is_some()
            && route.click_target.is_none();
        self.apply_pointer_component_state(&route, focus_before_dispatch)?;
        self.apply_pointer_transient_state_dirty(&route, pressed_before_dispatch)?;
        result.component_events = self.pointer_component_events(&route, &event)?;
        let range_action =
            self.apply_default_range_pointer_actions(&route, &mut result.component_events)?;
        if let Some(node_id) = range_action.captured_by {
            result.captured_by = Some(node_id);
            result.handled_by = Some(node_id);
            result.diagnostics.capture_started = true;
        }
        if let Some(node_id) = range_action.released_capture {
            result.released_capture = Some(node_id);
            result.handled_by = Some(node_id);
            result.diagnostics.capture_released = true;
        }
        if let Some(node_id) = range_action.handled_by {
            result.handled_by = Some(node_id);
        } else {
            let scrollbar_action = self.apply_default_scrollbar_pointer_action(&route)?;
            if let Some(node_id) = scrollbar_action.handled_by {
                result.handled_by = Some(node_id);
                result.diagnostics.scroll_defaulted = true;
            } else {
                self.apply_default_pointer_component_actions(
                    &route,
                    event.click_count,
                    &mut result.component_events,
                )?;
            }
            if let Some(node_id) = scrollbar_action.damage_node {
                self.push_damage_frame(&mut result, node_id);
            }
        }
        if let Some(node_id) = range_action.damage_node {
            self.push_damage_frame(&mut result, node_id);
        }
        self.push_focus_component_events(
            &mut result.component_events,
            focus_before_dispatch,
            self.focus.focused,
        )?;
        self.push_state_damage_frames(&mut result, &route, focus_before_dispatch);
        result.diagnostics.component_event_count = result.component_events.len();
        Ok(result)
    }

    fn apply_pointer_component_state(
        &mut self,
        route: &UiPointerRoute,
        focus_before_dispatch: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        for node_id in &route.entered {
            if self.component_states.set_hovered(*node_id, true) {
                self.mark_component_state_render_dirty(*node_id)?;
            }
        }
        for node_id in &route.left {
            if self.component_states.set_hovered(*node_id, false) {
                self.mark_component_state_render_dirty(*node_id)?;
            }
        }
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                if let Some(target) = route.target {
                    if self.node_interaction_enabled(target)?
                        && self.component_states.set_pressed(target, true)
                    {
                        self.mark_component_state_render_dirty(target)?;
                    }
                }
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(pressed) = route.pressed {
                    if self.component_states.set_pressed(pressed, false) {
                        self.mark_component_state_render_dirty(pressed)?;
                    }
                }
            }
            _ => {}
        }
        if focus_before_dispatch == self.focus.focused {
            return Ok(());
        }
        if let Some(previous) = focus_before_dispatch {
            if self.component_states.set_focused(previous, false) {
                self.mark_component_state_render_dirty(previous)?;
            }
        }
        if let Some(current) = self.focus.focused {
            if self.component_states.set_focused(current, true) {
                self.mark_component_state_render_dirty(current)?;
            }
        }
        Ok(())
    }

    fn apply_pointer_transient_state_dirty(
        &mut self,
        route: &UiPointerRoute,
        pressed_before_dispatch: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                if let Some(previous) =
                    pressed_before_dispatch.filter(|previous| Some(*previous) != route.target)
                {
                    self.set_node_pressed_dirty(previous, false)?;
                }
                if let Some(target) = route.target {
                    if self.node_interaction_enabled(target)? {
                        self.set_node_pressed_dirty(target, true)?;
                    }
                }
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(pressed) = route.pressed {
                    self.set_node_pressed_dirty(pressed, false)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn set_node_pressed_dirty(
        &mut self,
        node_id: UiNodeId,
        pressed: bool,
    ) -> Result<(), UiTreeError> {
        let component_state_changed = self.component_states.set_pressed(node_id, pressed);
        let state_flags_changed = {
            let node = self
                .tree
                .nodes
                .get_mut(&node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if node.state_flags.pressed == pressed {
                false
            } else {
                node.state_flags.pressed = pressed;
                true
            }
        };
        if component_state_changed || state_flags_changed {
            self.mark_component_state_render_dirty(node_id)?;
        }
        Ok(())
    }

    pub(crate) fn mark_component_state_render_dirty(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<(), UiTreeError> {
        let _ = self.apply_runtime_state_style_subtree(node_id, true)?;
        self.mark_node_dirty(
            node_id,
            UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
        )
    }

    fn pointer_component_events(
        &self,
        route: &UiPointerRoute,
        event: &UiPointerEvent,
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
                if self.node_interaction_enabled(node_id)? {
                    self.push_pointer_component_events(
                        &mut events,
                        node_id,
                        UiEventKind::Press,
                        UiComponentEvent::Press { pressed: true },
                        UiPointerComponentEventReason::PressBegin,
                    )?;
                }
            }
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryRelease {
            if let Some(node_id) = route.pressed {
                if self.node_interaction_enabled(node_id)? {
                    self.push_pointer_component_events(
                        &mut events,
                        node_id,
                        UiEventKind::Release,
                        UiComponentEvent::Press { pressed: false },
                        UiPointerComponentEventReason::PressEnd,
                    )?;
                }
            }
            if let Some(node_id) = route.click_target {
                if self.node_interaction_enabled(node_id)?
                    && !self.uses_typed_default_click_action(node_id)?
                {
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
                    if event.click_count >= 2 {
                        self.push_pointer_component_events(
                            &mut events,
                            node_id,
                            UiEventKind::DoubleClick,
                            UiComponentEvent::Commit {
                                property: "double_activated".to_string(),
                                value: zircon_runtime_interface::ui::component::UiValue::Bool(true),
                            },
                            UiPointerComponentEventReason::DefaultDoubleClick,
                        )?;
                    }
                }
            }
        }

        Ok(events)
    }

    fn push_focus_component_events(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        old_focus: Option<UiNodeId>,
        new_focus: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        if old_focus == new_focus {
            return Ok(());
        }
        if let Some(node_id) = old_focus {
            self.push_pointer_component_events(
                events,
                node_id,
                UiEventKind::Blur,
                UiComponentEvent::Focus { focused: false },
                UiPointerComponentEventReason::FocusLost,
            )?;
        }
        if let Some(node_id) = new_focus {
            self.push_pointer_component_events(
                events,
                node_id,
                UiEventKind::Focus,
                UiComponentEvent::Focus { focused: true },
                UiPointerComponentEventReason::FocusGained,
            )?;
        }
        Ok(())
    }

    fn push_state_damage_frames(
        &self,
        result: &mut UiPointerDispatchResult,
        route: &UiPointerRoute,
        focus_before_dispatch: Option<UiNodeId>,
    ) {
        for node_id in route.entered.iter().chain(route.left.iter()) {
            self.push_damage_frame(result, *node_id);
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryPress {
            if let Some(node_id) = route.target {
                self.push_damage_frame(result, node_id);
            }
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryRelease {
            if let Some(node_id) = route.pressed {
                self.push_damage_frame(result, node_id);
            }
            if let Some(node_id) = route.click_target {
                self.push_damage_frame(result, node_id);
            }
        }
        if focus_before_dispatch != self.focus.focused {
            if let Some(node_id) = focus_before_dispatch {
                self.push_damage_frame(result, node_id);
            }
            if let Some(node_id) = self.focus.focused {
                self.push_damage_frame(result, node_id);
            }
        }
    }

    fn push_damage_frame(&self, result: &mut UiPointerDispatchResult, node_id: UiNodeId) {
        let Some(frame) = self.arranged_tree.get(node_id).map(|node| node.frame) else {
            return;
        };
        if !result.requested_damage.contains(&frame) {
            result.requested_damage.push(frame);
        }
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
                self.focus_node_with_reason(
                    focus_target,
                    UiFocusChangeReason::Input,
                    UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction),
                )?;
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
            if let Some(owner) = captured {
                self.input.clear_pointer_capture_for(owner);
            } else {
                self.input.clear_pointer_capture();
            }
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
        if let Some(target) = route.target {
            if let Some(action) = default_interactions::range_navigation_action(kind) {
                let range_action = match action {
                    default_interactions::UiDefaultRangeNavigationAction::Step(direction) => {
                        self.mutate_default_range_step_value(target, direction)?
                    }
                    default_interactions::UiDefaultRangeNavigationAction::Minimum => {
                        self.mutate_default_range_endpoint_value(target, false)?
                    }
                    default_interactions::UiDefaultRangeNavigationAction::Maximum => {
                        self.mutate_default_range_endpoint_value(target, true)?
                    }
                };
                if range_action.is_some() {
                    let mut result = UiNavigationDispatchResult::new(route);
                    result.handled_by = Some(target);
                    return Ok(result);
                }
            }
        }
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if result.focus_changed_to.is_none() {
            if let Some(node_id) = self.tree.next_navigation_target(route.target, route.kind)? {
                result.handled_by = Some(route.target.unwrap_or(node_id));
                result.focus_changed_to = Some(node_id);
            }
        }
        if let Some(node_id) = result.focus_changed_to {
            self.focus_node_with_reason(
                node_id,
                UiFocusChangeReason::Navigation,
                UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation),
            )?;
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

fn bool_attribute(values: &std::collections::BTreeMap<String, toml::Value>, key: &str) -> bool {
    values.get(key).and_then(toml::Value::as_bool) == Some(true)
}

fn focus_reconcile_reason(property: &str, tree: &UiTree, node_id: UiNodeId) -> UiFocusChangeReason {
    match property {
        "enabled" | "focusable" => UiFocusChangeReason::Disabled,
        "visible" => UiFocusChangeReason::Hidden,
        "visibility" => tree
            .nodes
            .get(&node_id)
            .map(|node| {
                if node.is_render_visible() {
                    UiFocusChangeReason::Disabled
                } else {
                    UiFocusChangeReason::Hidden
                }
            })
            .unwrap_or(UiFocusChangeReason::Hidden),
        _ => UiFocusChangeReason::Clear,
    }
}

fn property_may_affect_runtime_pseudo_state(property: &str) -> bool {
    matches!(
        property,
        "checked"
            | "selected"
            | "disabled"
            | "enabled"
            | "pressed"
            | "active"
            | "expanded"
            | "popup_open"
            | "open"
            | "focus"
            | "focused"
            | "hover"
            | "hovered"
    )
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

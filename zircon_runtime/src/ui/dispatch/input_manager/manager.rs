use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiInputDispatchResult, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiPointerId, UiPointerInputEvent, UiPointerSource,
        UiSubmenuHoverTimerInputEvent, UiToastTimerInputEvent, UiTooltipTimerInputEvent,
        UiTooltipTimerInputEventKind, UiTypeaheadTimerInputEvent,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
    tree::UiTreeError,
    window::{UiWindowInputPumpBatch, UiWindowInputPumpEvent},
};

use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::{input, UiSurface},
};

use super::{
    outcome::UiInputDispatchOutcome, pointer_table::UiActivePointerTable, timers::UiInputTimerState,
};

#[derive(Default)]
pub struct UiInputManager {
    pointer: UiPointerDispatcher,
    navigation: UiNavigationDispatcher,
    pointers: UiActivePointerTable,
    timers: UiInputTimerState,
}

impl UiInputManager {
    pub fn pointer_dispatcher(&self) -> &UiPointerDispatcher {
        &self.pointer
    }

    pub fn pointer_dispatcher_mut(&mut self) -> &mut UiPointerDispatcher {
        &mut self.pointer
    }

    pub fn navigation_dispatcher(&self) -> &UiNavigationDispatcher {
        &self.navigation
    }

    pub fn navigation_dispatcher_mut(&mut self) -> &mut UiNavigationDispatcher {
        &mut self.navigation
    }

    pub fn active_pointers(&self) -> &UiActivePointerTable {
        &self.pointers
    }

    pub fn active_pointers_mut(&mut self) -> &mut UiActivePointerTable {
        &mut self.pointers
    }

    pub fn timers(&self) -> &UiInputTimerState {
        &self.timers
    }

    pub fn dispatch_input_event(
        &mut self,
        surface: &mut UiSurface,
        mut event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        let active_pointer_event = self.active_pointer_event_for_input(&event);
        apply_primary_touch_mouse_semantics(&mut event, active_pointer_event);
        self.clear_tooltip_for_activity(surface, &event);
        let pointer_release = self.prepare_double_click_pointer_release(surface, &mut event);
        let timestamp = input_event_timestamp(&event);
        let result = input::dispatch_input_event(surface, &self.pointer, &self.navigation, event)?;
        self.arm_double_click_from_pointer_release(pointer_release);
        self.update_active_pointer_table(surface, &result, active_pointer_event);
        self.arm_timers_from_component_events(surface, timestamp, &result);
        Ok(result)
    }

    pub fn dispatch_window_input_pump_event(
        &mut self,
        surface: &mut UiSurface,
        event: UiWindowInputPumpEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        match event {
            UiWindowInputPumpEvent::Input(input) => self.dispatch_input_event(surface, input),
            UiWindowInputPumpEvent::Window(window) => {
                input::dispatch_window_event(surface, &self.pointer, &self.navigation, window)
            }
        }
    }

    pub fn dispatch_window_input_pump_batch(
        &mut self,
        surface: &mut UiSurface,
        batch: UiWindowInputPumpBatch,
    ) -> Result<UiInputDispatchOutcome, UiTreeError> {
        let mut results = Vec::with_capacity(batch.events.len());
        for event in batch.events {
            results.push(self.dispatch_window_input_pump_event(surface, event)?);
        }
        Ok(UiInputDispatchOutcome::from_results(surface, results))
    }

    pub fn tick(
        &mut self,
        surface: &mut UiSurface,
        now: UiInputTimestamp,
    ) -> Result<Vec<UiInputDispatchResult>, UiTreeError> {
        self.timers.record_tick(now);
        self.timers.expire_double_click_candidate(now);
        let mut results = Vec::new();
        for target in self.timers.drain_expired_typeahead(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            results.push(input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::TypeaheadTimer(UiTypeaheadTimerInputEvent { metadata, target }),
            )?);
        }
        for (target, option_id) in self.timers.drain_expired_submenu_hover(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            results.push(input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::SubmenuHoverTimer(UiSubmenuHoverTimerInputEvent {
                    metadata,
                    target,
                    option_id,
                }),
            )?);
        }
        for (target, tooltip_id) in self.timers.drain_expired_tooltips(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            results.push(input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
                    metadata,
                    kind: UiTooltipTimerInputEventKind::Elapsed,
                    tooltip_id,
                    owner: Some(target),
                }),
            )?);
        }
        for (target, toast_id) in self.timers.drain_expired_toasts(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            results.push(input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::ToastTimer(UiToastTimerInputEvent {
                    metadata,
                    target,
                    toast_id,
                }),
            )?);
        }
        Ok(results)
    }

    fn arm_timers_from_component_events(
        &mut self,
        surface: &mut UiSurface,
        timestamp: UiInputTimestamp,
        result: &UiInputDispatchResult,
    ) {
        for report in &result.component_events {
            if !report.delivered {
                continue;
            }
            if matches!(report.event, UiComponentEvent::KeyboardText { .. }) {
                if let Some(timeout_ms) =
                    surface.typeahead_timeout_ms_for_component_node(report.target)
                {
                    self.timers
                        .arm_typeahead_expiration(report.target, timestamp, timeout_ms);
                }
            }
            self.arm_submenu_hover_timer_from_component_event(
                surface,
                timestamp,
                report.target,
                &report.event,
            );
            self.arm_tooltip_timer_from_component_event(
                surface,
                timestamp,
                report.target,
                &report.event,
            );
            self.arm_toast_timer_from_component_event(
                surface,
                timestamp,
                report.target,
                &report.event,
            );
        }
    }

    fn arm_submenu_hover_timer_from_component_event(
        &mut self,
        surface: &UiSurface,
        timestamp: UiInputTimestamp,
        target: UiNodeId,
        event: &UiComponentEvent,
    ) {
        let UiComponentEvent::ValueChanged { property, value } = event else {
            return;
        };
        if property != "hovered_option_id" {
            return;
        }

        let Some(delay_ms) = surface.submenu_hover_delay_ms_for_component_node(target) else {
            self.timers.clear_submenu_hover_expiration(target);
            return;
        };
        let option_id = match value {
            UiValue::String(value) | UiValue::Enum(value) if !value.is_empty() => value,
            _ => {
                self.timers.clear_submenu_hover_expiration(target);
                return;
            }
        };
        self.timers
            .arm_submenu_hover_expiration(target, option_id.as_str(), timestamp, delay_ms);
    }

    fn arm_tooltip_timer_from_component_event(
        &mut self,
        surface: &mut UiSurface,
        timestamp: UiInputTimestamp,
        target: UiNodeId,
        event: &UiComponentEvent,
    ) {
        match event {
            UiComponentEvent::Hover { hovered: true } => {
                let Some((tooltip_id, delay_ms)) = surface.tooltip_timer_for_component_node(target)
                else {
                    self.timers.clear_tooltip_expiration(target);
                    clear_tooltip_candidate_for_owner(surface, target);
                    return;
                };
                surface.input.arm_tooltip(tooltip_id.clone(), Some(target));
                self.timers
                    .arm_tooltip_expiration(target, tooltip_id, timestamp, delay_ms);
            }
            UiComponentEvent::Hover { hovered: false } => {
                self.timers.clear_tooltip_expiration(target);
                clear_tooltip_candidate_for_owner(surface, target);
            }
            _ => {}
        }
    }

    fn arm_toast_timer_from_component_event(
        &mut self,
        surface: &UiSurface,
        timestamp: UiInputTimestamp,
        target: UiNodeId,
        event: &UiComponentEvent,
    ) {
        match event {
            UiComponentEvent::ValueChanged { property, value }
                if matches!(property.as_str(), "toast_queue" | "queue") =>
            {
                if let Some((toast_id, timeout_ms)) = toast_timer_from_queue_value(value) {
                    self.timers
                        .arm_toast_expiration(target, toast_id, timestamp, timeout_ms);
                    return;
                }
                self.arm_toast_timer_from_surface_state(surface, timestamp, target);
            }
            UiComponentEvent::ValueChanged { property, .. }
                if matches!(
                    property.as_str(),
                    "current_toast_id"
                        | "auto_hide_duration_ms"
                        | "autoHideDuration"
                        | "message"
                        | "text"
                        | "open"
                        | "popup_open"
                ) =>
            {
                self.arm_toast_timer_from_surface_state(surface, timestamp, target);
            }
            UiComponentEvent::OpenPopup => {
                self.arm_toast_timer_from_surface_state(surface, timestamp, target);
            }
            UiComponentEvent::ClosePopup => {
                self.timers.clear_toast_expiration(target);
            }
            _ => {}
        }
    }

    fn arm_toast_timer_from_surface_state(
        &mut self,
        surface: &UiSurface,
        timestamp: UiInputTimestamp,
        target: UiNodeId,
    ) {
        let Some((toast_id, timeout_ms)) = surface.toast_timer_for_component_node(target) else {
            self.timers.clear_toast_expiration(target);
            return;
        };
        self.timers
            .arm_toast_expiration(target, toast_id, timestamp, timeout_ms);
    }

    fn clear_tooltip_for_activity(&mut self, surface: &mut UiSurface, event: &UiInputEvent) {
        if !input_event_cancels_tooltip(event) {
            return;
        }
        self.timers.clear_tooltip_expirations();
        surface.input.dismiss_transient_ui(
            zircon_runtime_interface::ui::dispatch::UiTransientDismissalTarget::Tooltip,
        );
    }

    fn prepare_double_click_pointer_release(
        &self,
        surface: &UiSurface,
        event: &mut UiInputEvent,
    ) -> Option<UiDoubleClickPointerRelease> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        let click_target = pointer_release_click_target(surface, pointer)?;
        let click_count = self.timers.double_click_count_for_release(
            click_target,
            pointer.metadata.pointer_id,
            pointer.metadata.pointer_source,
            pointer.event.button,
            pointer.metadata.timestamp,
        );
        pointer.event.click_count = pointer.event.click_count.max(click_count);
        Some(UiDoubleClickPointerRelease {
            target: click_target,
            pointer_id: pointer.metadata.pointer_id,
            pointer_source: pointer.metadata.pointer_source,
            button: pointer.event.button,
            timestamp: pointer.metadata.timestamp,
            click_count: pointer.event.click_count,
        })
    }

    fn arm_double_click_from_pointer_release(
        &mut self,
        pointer_release: Option<UiDoubleClickPointerRelease>,
    ) {
        let Some(pointer_release) = pointer_release else {
            return;
        };
        self.timers.arm_double_click_candidate(
            pointer_release.target,
            pointer_release.pointer_id,
            pointer_release.pointer_source,
            pointer_release.button,
            pointer_release.click_count,
            pointer_release.timestamp,
        );
    }

    fn active_pointer_event_for_input(
        &self,
        event: &UiInputEvent,
    ) -> Option<UiActivePointerInputEvent> {
        let UiInputEvent::Pointer(pointer) = event else {
            return None;
        };
        let pointer_id = pointer.metadata.pointer_id.unwrap_or_default();
        Some(UiActivePointerInputEvent {
            pointer_id,
            source: pointer.metadata.pointer_source,
            kind: pointer.event.kind,
            point: pointer.event.point,
            button: pointer.event.button,
            is_primary: self.active_pointer_is_primary(pointer_id, pointer.metadata.pointer_source),
        })
    }

    fn update_active_pointer_table(
        &mut self,
        surface: &UiSurface,
        result: &UiInputDispatchResult,
        active_pointer_event: Option<UiActivePointerInputEvent>,
    ) {
        let Some(active_pointer_event) = active_pointer_event else {
            return;
        };
        let pointer_id = active_pointer_event.pointer_id;
        if matches!(active_pointer_event.kind, UiPointerEventKind::Cancel) {
            self.pointers.remove(pointer_id);
            return;
        }

        self.pointers.upsert(
            pointer_id,
            active_pointer_event.source,
            active_pointer_event.is_primary,
        );
        self.pointers
            .record_point(pointer_id, active_pointer_event.point);
        self.pointers
            .set_hovered_path(pointer_id, active_pointer_hover_path(result));
        match active_pointer_event.kind {
            UiPointerEventKind::Down => {
                self.pointers.press_button(
                    pointer_id,
                    active_pointer_event.button,
                    result.diagnostics.route_target,
                );
            }
            UiPointerEventKind::Up => {
                self.pointers
                    .release_button(pointer_id, active_pointer_event.button);
            }
            UiPointerEventKind::Move | UiPointerEventKind::Scroll | UiPointerEventKind::Cancel => {}
        }
        let capture_target = surface
            .input
            .pointer_capture_owner(pointer_id)
            .or(result.diagnostics.route_trace.capture_target);
        self.pointers.set_capture_target(pointer_id, capture_target);
    }

    fn active_pointer_is_primary(&self, pointer_id: UiPointerId, source: UiPointerSource) -> bool {
        if let Some(entry) = self.pointers.entry(pointer_id) {
            return entry.is_primary;
        }
        !source.is_touch_like()
            || !self
                .pointers
                .entries()
                .iter()
                .any(|entry| entry.source == source && entry.is_primary)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct UiActivePointerInputEvent {
    pointer_id: UiPointerId,
    source: UiPointerSource,
    kind: UiPointerEventKind,
    point: UiPoint,
    button: Option<UiPointerButton>,
    is_primary: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiDoubleClickPointerRelease {
    target: UiNodeId,
    pointer_id: Option<UiPointerId>,
    pointer_source: UiPointerSource,
    button: Option<UiPointerButton>,
    timestamp: UiInputTimestamp,
    click_count: u8,
}

fn apply_primary_touch_mouse_semantics(
    event: &mut UiInputEvent,
    active_pointer_event: Option<UiActivePointerInputEvent>,
) {
    let Some(active_pointer_event) = active_pointer_event else {
        return;
    };
    if active_pointer_event.is_primary || !active_pointer_event.source.is_touch_like() {
        return;
    }
    let UiInputEvent::Pointer(pointer) = event else {
        return;
    };
    pointer.event.button = None;
}

fn input_event_cancels_tooltip(event: &UiInputEvent) -> bool {
    matches!(
        event,
        UiInputEvent::Pointer(_)
            | UiInputEvent::Keyboard(_)
            | UiInputEvent::Text(_)
            | UiInputEvent::Ime(_)
            | UiInputEvent::Navigation(_)
            | UiInputEvent::Analog(_)
            | UiInputEvent::MouseMotion(_)
            | UiInputEvent::DragDrop(_)
            | UiInputEvent::Accessibility(_)
    )
}

fn pointer_release_click_target(
    surface: &UiSurface,
    pointer: &UiPointerInputEvent,
) -> Option<UiNodeId> {
    if !matches!(pointer.event.kind, UiPointerEventKind::Up)
        || pointer.event.button != Some(UiPointerButton::Primary)
    {
        return None;
    }
    let pressed = surface.focus.pressed?;
    let hit = surface.hit_test(pointer.event.point);
    hit.stacked.contains(&pressed).then_some(pressed)
}

fn active_pointer_hover_path(result: &UiInputDispatchResult) -> Vec<UiNodeId> {
    if !result.diagnostics.route_trace.bubble_path.is_empty() {
        return result.diagnostics.route_trace.bubble_path.clone();
    }
    result
        .diagnostics
        .route_trace
        .direct_target
        .or(result.diagnostics.route_target)
        .into_iter()
        .collect()
}

fn clear_tooltip_candidate_for_owner(surface: &mut UiSurface, target: UiNodeId) {
    let Some(tooltip_id) = surface
        .input
        .tooltip
        .as_ref()
        .filter(|tooltip| tooltip.owner == Some(target))
        .map(|tooltip| tooltip.tooltip_id.clone())
    else {
        return;
    };
    surface.input.clear_tooltip(tooltip_id.as_str());
}

fn toast_timer_from_queue_value(value: &UiValue) -> Option<(String, u64)> {
    match value {
        UiValue::Array(values) => values.iter().find_map(toast_timer_from_queue_value),
        UiValue::Map(values) => {
            let toast_id =
                first_string_value(values, &["id", "toast_id", "toastId", "value", "key"])?;
            let timeout_ms = first_u64_value(
                values,
                &[
                    "duration",
                    "duration_ms",
                    "auto_hide_duration_ms",
                    "autoHideDuration",
                ],
            )?;
            (timeout_ms > 0).then_some((toast_id, timeout_ms))
        }
        UiValue::String(value) | UiValue::Enum(value) => toast_timer_from_queue_string(value),
        _ => None,
    }
}

fn toast_timer_from_queue_string(value: &str) -> Option<(String, u64)> {
    let mut parts = value.split('|');
    let toast_id = parts.next()?.trim().to_string();
    if toast_id.is_empty() {
        return None;
    }

    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        if matches!(
            key.trim(),
            "duration" | "duration_ms" | "auto_hide_duration_ms" | "autoHideDuration"
        ) {
            let timeout_ms = value.trim().parse::<u64>().ok()?;
            return (timeout_ms > 0).then_some((toast_id, timeout_ms));
        }
    }
    None
}

fn first_string_value(
    values: &std::collections::BTreeMap<String, UiValue>,
    keys: &[&str],
) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn first_u64_value(
    values: &std::collections::BTreeMap<String, UiValue>,
    keys: &[&str],
) -> Option<u64> {
    keys.iter()
        .find_map(|key| values.get(*key).and_then(u64_value))
        .filter(|value| *value > 0)
}

fn string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    }
}

fn u64_value(value: &UiValue) -> Option<u64> {
    match value {
        UiValue::Int(value) => Some((*value).max(0) as u64),
        UiValue::Float(value) => Some((*value).round().max(0.0) as u64),
        UiValue::String(value) | UiValue::Enum(value) => value.parse::<u64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zircon_runtime_interface::ui::{
        component::{UiComponentEvent, UiValue},
        dispatch::{
            UiComponentEventReport, UiDispatchDisposition, UiDispatchReply, UiInputDispatchResult,
            UiInputEvent, UiInputEventMetadata, UiInputRoutePolicy, UiInputSequence,
            UiInputTimestamp, UiTextInputEvent,
        },
        dispatch::{UiDispatchHostRequestKind, UiTooltipTimerInputEventKind},
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        tree::{UiTemplateNodeMetadata, UiTreeNode},
        widget::UiWidgetContract,
    };

    use crate::ui::surface::UiSurface;

    use super::UiInputManager;

    #[test]
    fn hovered_menu_option_arms_replaces_and_clears_submenu_hover_timer() {
        let target = UiNodeId::new(2);
        for component in ["MenuList", "ContextMenu", "DropdownPopup"] {
            let mut surface = submenu_hover_surface(component);
            let mut manager = UiInputManager::default();

            manager.arm_timers_from_component_events(
                &mut surface,
                UiInputTimestamp::from_micros(50),
                &hover_changed_result(target, "file"),
            );

            assert_eq!(
                manager.timers().submenu_hover_expiration(target),
                Some(UiInputTimestamp::from_micros(80_050)),
                "{component} should arm submenu hover from hovered_option_id"
            );
            assert_eq!(
                manager.timers().submenu_hover_option_id(target),
                Some("file"),
                "{component} should retain the hovered submenu option id"
            );

            manager.arm_timers_from_component_events(
                &mut surface,
                UiInputTimestamp::from_micros(70),
                &hover_changed_result(target, "edit"),
            );

            assert_eq!(
                manager.timers().submenu_hover_expiration(target),
                Some(UiInputTimestamp::from_micros(80_070)),
                "{component} should replace an existing submenu hover timer"
            );
            assert_eq!(
                manager.timers().submenu_hover_option_id(target),
                Some("edit"),
                "{component} should replace the pending submenu option id"
            );

            manager.arm_timers_from_component_events(
                &mut surface,
                UiInputTimestamp::from_micros(90),
                &hover_changed_result(target, ""),
            );

            assert_eq!(
                manager.timers().submenu_hover_expiration(target),
                None,
                "{component} should clear submenu hover when hover leaves an option"
            );
            assert_eq!(manager.timers().submenu_hover_option_id(target), None);
        }
    }

    #[test]
    fn popup_menu_shells_expose_typeahead_and_submenu_timer_contracts() {
        let target = UiNodeId::new(2);
        for component in ["MenuList", "ContextMenu", "DropdownPopup"] {
            let surface = submenu_hover_surface(component);
            assert_eq!(
                surface.typeahead_timeout_ms_for_component_node(target),
                Some(120),
                "{component} should use authored typeahead timing"
            );
            assert_eq!(
                surface.submenu_hover_delay_ms_for_component_node(target),
                Some(80),
                "{component} should use authored submenu hover timing"
            );
        }
    }

    #[test]
    fn toast_queue_value_arms_replaces_and_clears_auto_hide_timer() {
        let target = UiNodeId::new(2);
        let mut surface = toast_surface("surface-save", 4000);
        let mut manager = UiInputManager::default();

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(50),
            &component_event_result(
                target,
                UiComponentEvent::ValueChanged {
                    property: "toast_queue".to_string(),
                    value: UiValue::String("save|message=Saved|autoHideDuration=40".to_string()),
                },
            ),
        );

        assert_eq!(
            manager.timers().toast_expiration(target),
            Some(UiInputTimestamp::from_micros(40_050))
        );
        assert_eq!(manager.timers().toast_id(target), Some("save"));

        let mut next_toast = BTreeMap::new();
        next_toast.insert("id".to_string(), UiValue::String("export".to_string()));
        next_toast.insert("auto_hide_duration_ms".to_string(), UiValue::Int(80));
        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(70),
            &component_event_result(
                target,
                UiComponentEvent::ValueChanged {
                    property: "toast_queue".to_string(),
                    value: UiValue::Array(vec![UiValue::Map(next_toast)]),
                },
            ),
        );

        assert_eq!(
            manager.timers().toast_expiration(target),
            Some(UiInputTimestamp::from_micros(80_070))
        );
        assert_eq!(manager.timers().toast_id(target), Some("export"));

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(90),
            &component_event_result(target, UiComponentEvent::ClosePopup),
        );

        assert_eq!(manager.timers().toast_expiration(target), None);
        assert_eq!(manager.timers().toast_id(target), None);
    }

    #[test]
    fn toast_auto_hide_tick_dispatches_expired_commit_event() {
        let target = UiNodeId::new(2);
        let mut surface = toast_surface("save", 40);
        let mut manager = UiInputManager::default();

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(10),
            &component_event_result(
                target,
                UiComponentEvent::ValueChanged {
                    property: "current_toast_id".to_string(),
                    value: UiValue::String("save".to_string()),
                },
            ),
        );

        assert_eq!(
            manager.timers().toast_expiration(target),
            Some(UiInputTimestamp::from_micros(40_010))
        );

        let early = manager
            .tick(&mut surface, UiInputTimestamp::from_micros(40_009))
            .unwrap();
        assert!(early.is_empty());

        let expired = manager
            .tick(&mut surface, UiInputTimestamp::from_micros(40_010))
            .unwrap();

        assert_eq!(expired.len(), 1);
        let expired = &expired[0];
        assert_eq!(expired.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(expired.reply.handler, Some(target));
        assert_eq!(
            expired.diagnostics.handled_phase.as_deref(),
            Some("toast_timer.component_event")
        );
        assert_eq!(
            expired.diagnostics.route_policy,
            UiInputRoutePolicy::DefaultAction
        );
        assert_eq!(expired.diagnostics.route_target, Some(target));
        assert_eq!(expired.component_events.len(), 1);
        assert_eq!(expired.component_events[0].target, target);
        assert_eq!(
            expired.component_events[0].event,
            UiComponentEvent::Commit {
                property: "expired_toast_id".to_string(),
                value: UiValue::String("save".to_string()),
            }
        );
        match &expired.event {
            UiInputEvent::ToastTimer(timer) => {
                assert_eq!(
                    timer.metadata.timestamp,
                    UiInputTimestamp::from_micros(40_010)
                );
                assert_eq!(timer.target, target);
                assert_eq!(timer.toast_id, "save");
            }
            other => panic!("expected toast timer input event, got {other:?}"),
        }
        assert_eq!(manager.timers().toast_expiration(target), None);
    }

    #[test]
    fn tooltip_hover_arms_and_clears_manager_timer_candidate() {
        let target = UiNodeId::new(2);
        let mut surface = tooltip_surface("status.hint", 40);
        let mut manager = UiInputManager::default();

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(25),
            &component_event_result(target, UiComponentEvent::Hover { hovered: true }),
        );

        assert_eq!(
            manager.timers().tooltip_expiration(target),
            Some(UiInputTimestamp::from_micros(40_025))
        );
        assert_eq!(manager.timers().tooltip_id(target), Some("status.hint"));
        assert_eq!(
            surface.input.tooltip.as_ref().map(|tooltip| (
                tooltip.tooltip_id.as_str(),
                tooltip.owner,
                tooltip.visible
            )),
            Some(("status.hint", Some(target), false))
        );

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(30),
            &component_event_result(target, UiComponentEvent::Hover { hovered: false }),
        );

        assert_eq!(manager.timers().tooltip_expiration(target), None);
        assert_eq!(manager.timers().tooltip_id(target), None);
        assert_eq!(surface.input.tooltip, None);
    }

    #[test]
    fn tooltip_hover_timer_tick_dispatches_elapsed_default_action() {
        let target = UiNodeId::new(2);
        let mut surface = tooltip_surface("status.hint", 40);
        let mut manager = UiInputManager::default();

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(10),
            &component_event_result(target, UiComponentEvent::Hover { hovered: true }),
        );

        let early = manager
            .tick(&mut surface, UiInputTimestamp::from_micros(40_009))
            .unwrap();
        assert!(early.is_empty());

        let expired = manager
            .tick(&mut surface, UiInputTimestamp::from_micros(40_010))
            .unwrap();

        assert_eq!(expired.len(), 1);
        let expired = &expired[0];
        assert_eq!(expired.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(expired.diagnostics.route_target, Some(target));
        assert_eq!(
            expired.diagnostics.route_policy,
            UiInputRoutePolicy::DefaultAction
        );
        assert_eq!(
            expired.diagnostics.handled_phase.as_deref(),
            Some("tooltip.effect")
        );
        assert!(matches!(
            expired.host_requests[0].request,
            UiDispatchHostRequestKind::Tooltip {
                kind: zircon_runtime_interface::ui::dispatch::UiTooltipEffectKind::Show,
                ref tooltip_id,
            } if tooltip_id == "status.hint"
        ));
        assert_eq!(
            surface.input.tooltip.as_ref().map(|tooltip| (
                tooltip.tooltip_id.as_str(),
                tooltip.owner,
                tooltip.visible
            )),
            Some(("status.hint", Some(target), true))
        );
        match &expired.event {
            UiInputEvent::TooltipTimer(tooltip) => {
                assert_eq!(tooltip.kind, UiTooltipTimerInputEventKind::Elapsed);
                assert_eq!(tooltip.tooltip_id, "status.hint");
                assert_eq!(tooltip.owner, Some(target));
                assert!(tooltip.metadata.synthetic);
            }
            other => panic!("expected tooltip timer input event, got {other:?}"),
        }
        assert_eq!(manager.timers().tooltip_expiration(target), None);
    }

    #[test]
    fn tooltip_candidate_clears_on_following_input_activity() {
        let target = UiNodeId::new(2);
        let mut surface = tooltip_surface("status.hint", 40);
        let mut manager = UiInputManager::default();

        manager.arm_timers_from_component_events(
            &mut surface,
            UiInputTimestamp::from_micros(10),
            &component_event_result(target, UiComponentEvent::Hover { hovered: true }),
        );
        assert_eq!(manager.timers().tooltip_id(target), Some("status.hint"));
        assert!(surface.input.tooltip.is_some());

        manager
            .dispatch_input_event(
                &mut surface,
                UiInputEvent::Text(UiTextInputEvent {
                    metadata: UiInputEventMetadata::new(
                        UiInputTimestamp::from_micros(20),
                        UiInputSequence::new(20),
                    ),
                    text: "x".to_string(),
                }),
            )
            .unwrap();

        assert_eq!(manager.timers().tooltip_expiration(target), None);
        assert_eq!(manager.timers().tooltip_id(target), None);
        assert_eq!(surface.input.tooltip, None);
    }

    fn submenu_hover_surface(component: &str) -> UiSurface {
        let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.submenu_hover"));
        surface
            .tree
            .insert_root(UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("menu")));
        surface
            .tree
            .nodes
            .get_mut(&UiNodeId::new(2))
            .unwrap()
            .template_metadata = Some(UiTemplateNodeMetadata {
            component: component.to_string(),
            control_id: Some("SceneMenu".to_string()),
            attributes: toml::from_str(
                r#"
typeahead_timeout_ms = 120
submenu_hover_delay_ms = 80
"#,
            )
            .unwrap(),
            ..Default::default()
        });
        surface.rebuild();
        surface
    }

    fn toast_surface(toast_id: &str, duration_ms: i64) -> UiSurface {
        let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.toast"));
        surface
            .tree
            .insert_root(UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("toast")));
        let mut attributes = BTreeMap::new();
        attributes.insert(
            "current_toast_id".to_string(),
            toml::Value::String(toast_id.to_string()),
        );
        attributes.insert(
            "auto_hide_duration_ms".to_string(),
            toml::Value::Integer(duration_ms),
        );
        attributes.insert("open".to_string(), toml::Value::Boolean(true));
        surface
            .tree
            .nodes
            .get_mut(&UiNodeId::new(2))
            .unwrap()
            .template_metadata = Some(UiTemplateNodeMetadata {
            component: "Snackbar".to_string(),
            control_id: Some("StatusToast".to_string()),
            bindings: vec![binding("Snackbar/Commit", "Change")],
            attributes,
            ..Default::default()
        });
        surface.rebuild();
        surface
    }

    fn tooltip_surface(tooltip_id: &str, delay_ms: i64) -> UiSurface {
        let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.tooltip"));
        surface
            .tree
            .insert_root(UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("button")));
        let mut attributes = BTreeMap::new();
        attributes.insert(
            "tooltip_delay_ms".to_string(),
            toml::Value::Integer(delay_ms),
        );
        surface
            .tree
            .nodes
            .get_mut(&UiNodeId::new(2))
            .unwrap()
            .template_metadata = Some(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some("StatusButton".to_string()),
            widget: UiWidgetContract {
                tooltip: Some(tooltip_id.to_string()),
                ..UiWidgetContract::default()
            },
            attributes,
            ..Default::default()
        });
        surface.rebuild();
        surface
    }

    fn hover_changed_result(target: UiNodeId, option_id: &str) -> UiInputDispatchResult {
        component_event_result(
            target,
            UiComponentEvent::ValueChanged {
                property: "hovered_option_id".to_string(),
                value: UiValue::String(option_id.to_string()),
            },
        )
    }

    fn component_event_result(target: UiNodeId, event: UiComponentEvent) -> UiInputDispatchResult {
        let mut result = UiInputDispatchResult::new(
            UiInputEvent::Text(UiTextInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(0),
                    UiInputSequence::new(0),
                ),
                text: String::new(),
            }),
            UiDispatchReply::handled(),
        );
        result.component_events.push(UiComponentEventReport {
            target,
            event,
            delivered: true,
            drag: None,
        });
        result
    }

    fn binding(id: &str, event: &str) -> zircon_runtime_interface::ui::template::UiBindingRef {
        zircon_runtime_interface::ui::template::UiBindingRef {
            id: id.to_string(),
            event: match event {
                "Change" => zircon_runtime_interface::ui::binding::UiEventKind::Change,
                other => panic!("unsupported binding event {other}"),
            },
            route: Some(id.replace('/', ".")),
            action: None,
            targets: Vec::new(),
        }
    }
}

fn input_event_timestamp(event: &UiInputEvent) -> UiInputTimestamp {
    match event {
        UiInputEvent::Pointer(pointer) => pointer.metadata.timestamp,
        UiInputEvent::Keyboard(keyboard) => keyboard.metadata.timestamp,
        UiInputEvent::Text(text) => text.metadata.timestamp,
        UiInputEvent::Ime(ime) => ime.metadata.timestamp,
        UiInputEvent::Navigation(navigation) => navigation.metadata.timestamp,
        UiInputEvent::Analog(analog) => analog.metadata.timestamp,
        UiInputEvent::MouseMotion(motion) => motion.metadata.timestamp,
        UiInputEvent::DragDrop(drag_drop) => drag_drop.metadata.timestamp,
        UiInputEvent::Popup(popup) => popup.metadata.timestamp,
        UiInputEvent::TooltipTimer(tooltip) => tooltip.metadata.timestamp,
        UiInputEvent::TypeaheadTimer(typeahead) => typeahead.metadata.timestamp,
        UiInputEvent::SubmenuHoverTimer(submenu_hover) => submenu_hover.metadata.timestamp,
        UiInputEvent::ToastTimer(toast) => toast.metadata.timestamp,
        UiInputEvent::Accessibility(accessibility) => accessibility.metadata.timestamp,
    }
}

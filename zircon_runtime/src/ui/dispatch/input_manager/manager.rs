use std::time::Duration;

use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchPhase, UiDispatchReply, UiImeInputEvent, UiImeInputEventKind,
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
    surface::{UiSurface, input},
};

use super::{
    ime_host_requests::{
        append_ime_host_requests_for_input_method_requests, append_ime_host_requests_for_result,
    },
    outcome::UiInputDispatchOutcome,
    pointer_table::UiActivePointerTable,
    timers::UiInputTimerState,
};
use crate::core::framework::input::ImeHostRequest;

#[derive(Default)]
pub struct UiInputManager {
    pointer: UiPointerDispatcher,
    navigation: UiNavigationDispatcher,
    pointers: UiActivePointerTable,
    timers: UiInputTimerState,
    ime_host_requests: Vec<ImeHostRequest>,
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

    pub fn next_frame_visible_delay(&self, now: UiInputTimestamp) -> Option<Duration> {
        self.timers.next_frame_visible_delay(now)
    }

    pub fn drain_ime_host_requests(&mut self) -> Vec<ImeHostRequest> {
        std::mem::take(&mut self.ime_host_requests)
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
        self.record_ime_host_requests_from_result(&result);
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
                let result =
                    input::dispatch_window_event(surface, &self.pointer, &self.navigation, window)?;
                self.record_ime_host_requests_from_result(&result);
                Ok(result)
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
            let result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::TypeaheadTimer(UiTypeaheadTimerInputEvent { metadata, target }),
            )?;
            self.record_ime_host_requests_from_result(&result);
            results.push(result);
        }
        for (target, option_id) in self.timers.drain_expired_submenu_hover(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::SubmenuHoverTimer(UiSubmenuHoverTimerInputEvent {
                    metadata,
                    target,
                    option_id,
                }),
            )?;
            self.record_ime_host_requests_from_result(&result);
            results.push(result);
        }
        for (target, tooltip_id) in self.timers.drain_expired_tooltips(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
                    metadata,
                    kind: UiTooltipTimerInputEventKind::Elapsed,
                    tooltip_id,
                    owner: Some(target),
                }),
            )?;
            self.record_ime_host_requests_from_result(&result);
            results.push(result);
        }
        for (target, toast_id) in self.timers.drain_expired_toasts(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::ToastTimer(UiToastTimerInputEvent {
                    metadata,
                    target,
                    toast_id,
                }),
            )?;
            self.record_ime_host_requests_from_result(&result);
            results.push(result);
        }
        let lifecycle = surface.input.take_deferred_focus_input_lifecycle();
        append_ime_host_requests_for_input_method_requests(
            lifecycle.input_method_requests,
            &mut self.ime_host_requests,
        );
        if !lifecycle.component_events.is_empty() {
            let result = focus_input_method_lifecycle_result(now, lifecycle.component_events);
            self.arm_timers_from_component_events(surface, now, &result);
            results.push(result);
        }
        Ok(results)
    }

    fn record_ime_host_requests_from_result(&mut self, result: &UiInputDispatchResult) {
        append_ime_host_requests_for_result(result, &mut self.ime_host_requests);
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

fn focus_input_method_lifecycle_result(
    now: UiInputTimestamp,
    component_events: Vec<zircon_runtime_interface::ui::dispatch::UiComponentEventReport>,
) -> UiInputDispatchResult {
    let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
    metadata.synthetic = true;
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::Ime(UiImeInputEvent {
            metadata,
            kind: UiImeInputEventKind::Cancel,
            text: String::new(),
            cursor_range: None,
            delete_surrounding: None,
        }),
        UiDispatchReply::handled().in_phase(UiDispatchPhase::DefaultAction),
    );
    result.diagnostics.routed = true;
    result.diagnostics.handled_phase = Some("focus.input_method_lifecycle".to_string());
    result.component_events = component_events;
    result
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
mod tests;

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

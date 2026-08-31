use std::time::Duration;

use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchPhase, UiDispatchReply, UiImeInputEvent, UiImeInputEventKind,
        UiInputDiagnosticsMode, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata,
        UiInputSequence, UiInputTimestamp, UiPointerId, UiPointerInputEvent, UiPointerSource,
        UiSubmenuHoverTimerInputEvent, UiToastTimerInputEvent, UiTooltipTimerInputEvent,
        UiTooltipTimerInputEventKind, UiTypeaheadTimerInputEvent,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{UiHitTestQuery, UiPointerButton, UiPointerEventKind},
    tree::UiTreeError,
    window::{UiWindowInputPumpBatch, UiWindowInputPumpEvent},
};

use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::{input, UiSurface},
};

use super::{
    bound_text_model_updates::UiTextModelUpdateState,
    clipboard_host_requests::UiClipboardHostRequestQueue,
    ime_host_requests::{
        append_ime_host_requests_for_input_method_requests, append_ime_host_requests_for_result,
    },
    number_model_updates::UiNumberModelUpdateState,
    outcome::UiInputDispatchOutcome,
    pointer_table::UiActivePointerTable,
    text_document_session::UiTextDocumentSession,
    text_focus_lifecycle::finish_pending_text_focus_loss,
    timers::UiInputTimerState,
};
use crate::core::framework::input::ImeHostRequest;

pub struct UiInputManager {
    pointer: UiPointerDispatcher,
    navigation: UiNavigationDispatcher,
    diagnostics_mode: UiInputDiagnosticsMode,
    pointers: UiActivePointerTable,
    timers: UiInputTimerState,
    ime_host_requests: Vec<ImeHostRequest>,
    clipboard_host_requests: UiClipboardHostRequestQueue,
    pub(super) text_documents: UiTextDocumentSession,
    pub(super) text_model_updates: UiTextModelUpdateState,
    pub(super) number_model_updates: UiNumberModelUpdateState,
}

impl Default for UiInputManager {
    fn default() -> Self {
        Self {
            pointer: UiPointerDispatcher::default(),
            navigation: UiNavigationDispatcher::default(),
            diagnostics_mode: UiInputDiagnosticsMode::Full,
            pointers: UiActivePointerTable::default(),
            timers: UiInputTimerState::default(),
            ime_host_requests: Vec::new(),
            clipboard_host_requests: UiClipboardHostRequestQueue::default(),
            text_documents: UiTextDocumentSession::default(),
            text_model_updates: UiTextModelUpdateState::default(),
            number_model_updates: UiNumberModelUpdateState::default(),
        }
    }
}

impl UiInputManager {
    pub fn summary() -> Self {
        Self {
            diagnostics_mode: UiInputDiagnosticsMode::Summary,
            ..Self::default()
        }
    }

    pub fn diagnostics_mode(&self) -> UiInputDiagnosticsMode {
        self.diagnostics_mode
    }

    pub fn set_diagnostics_mode(&mut self, diagnostics_mode: UiInputDiagnosticsMode) {
        self.diagnostics_mode = diagnostics_mode;
    }

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

    pub fn tooltip_intro_progress(&self, now: UiInputTimestamp) -> Option<f32> {
        self.timers.tooltip_intro_progress(now)
    }

    /// Arms a tooltip whose owner and text were resolved by a host-specific presentation layer.
    ///
    /// The input manager remains the timing and transient-state authority even when the host uses
    /// richer metadata than the generic surface tooltip contract can express.
    pub fn arm_tooltip_candidate(
        &mut self,
        surface: &mut UiSurface,
        started_at: UiInputTimestamp,
        owner: UiNodeId,
        tooltip_id: impl Into<String>,
        delay_ms: u64,
    ) {
        let tooltip_id = tooltip_id.into();
        self.dismiss_tooltip(surface);
        surface.input.arm_tooltip(tooltip_id.clone(), Some(owner));
        self.timers
            .arm_tooltip_expiration(owner, tooltip_id, started_at, delay_ms);
    }

    /// Clears every pending or visible tooltip owned by this input manager.
    pub fn dismiss_tooltip(&mut self, surface: &mut UiSurface) {
        self.timers.clear_tooltip_expirations();
        self.timers.clear_tooltip_intro();
        surface.input.dismiss_transient_ui(
            zircon_runtime_interface::ui::dispatch::UiTransientDismissalTarget::Tooltip,
        );
    }

    pub fn drain_ime_host_requests(&mut self) -> Vec<ImeHostRequest> {
        std::mem::take(&mut self.ime_host_requests)
    }

    pub(crate) fn drain_clipboard_host_requests_into(
        &mut self,
        output: &mut Vec<zircon_runtime_interface::ui::dispatch::UiClipboardRequest>,
    ) {
        self.clipboard_host_requests.drain_into(output);
    }

    pub fn dispatch_input_event(
        &mut self,
        surface: &mut UiSurface,
        event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        self.dispatch_input_event_with_query(surface, event, None)
    }

    pub(crate) fn dispatch_input_event_with_query(
        &mut self,
        surface: &mut UiSurface,
        mut event: UiInputEvent,
        pointer_query: Option<UiHitTestQuery>,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        self.synchronize_text_document_owners(surface);
        let active_pointer_event = self.active_pointer_event_for_input(&event);
        apply_primary_touch_mouse_semantics(&mut event, active_pointer_event);
        self.clear_tooltip_for_activity(surface, &event);
        let pointer_release = self.prepare_double_click_pointer_release(surface, &mut event);
        let timestamp = input_event_timestamp(&event);
        let diagnostics_mode = self.diagnostics_mode;
        let mut result = input::dispatch_input_event(
            surface,
            &self.pointer,
            &self.navigation,
            event,
            pointer_query,
            Some(&mut self.text_documents),
            diagnostics_mode,
        )?;
        self.arm_double_click_from_pointer_release(pointer_release);
        self.update_active_pointer_table(surface, &result, active_pointer_event);
        self.arm_timers_from_component_events(surface, timestamp, &result);
        self.record_text_service_lifecycle_from_result(surface, &mut result);
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
                self.synchronize_text_document_owners(surface);
                let mut result = input::dispatch_window_event(
                    surface,
                    &self.pointer,
                    &self.navigation,
                    window,
                    self.diagnostics_mode,
                )?;
                self.record_text_service_lifecycle_from_result(surface, &mut result);
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
        self.synchronize_text_document_owners(surface);
        self.timers.record_tick(now);
        self.timers.expire_double_click_candidate(now);
        self.timers.expire_tooltip_intro(now);
        let mut results = Vec::new();
        for target in self.timers.drain_expired_typeahead(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let mut result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::TypeaheadTimer(UiTypeaheadTimerInputEvent { metadata, target }),
                None,
                None,
                self.diagnostics_mode,
            )?;
            self.record_text_service_lifecycle_from_result(surface, &mut result);
            results.push(result);
        }
        for (target, option_id) in self.timers.drain_expired_submenu_hover(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let mut result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::SubmenuHoverTimer(UiSubmenuHoverTimerInputEvent {
                    metadata,
                    target,
                    option_id,
                }),
                None,
                None,
                self.diagnostics_mode,
            )?;
            self.record_text_service_lifecycle_from_result(surface, &mut result);
            results.push(result);
        }
        for (target, tooltip_id) in self.timers.drain_expired_tooltips(now) {
            let intro_tooltip_id = tooltip_id.clone();
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let mut result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
                    metadata,
                    kind: UiTooltipTimerInputEventKind::Elapsed,
                    tooltip_id,
                    owner: Some(target),
                }),
                None,
                None,
                self.diagnostics_mode,
            )?;
            self.record_text_service_lifecycle_from_result(surface, &mut result);
            if surface.input.tooltip.as_ref().is_some_and(|tooltip| {
                tooltip.visible
                    && tooltip.owner == Some(target)
                    && tooltip.tooltip_id == intro_tooltip_id
            }) {
                self.timers.arm_tooltip_intro(now);
            }
            results.push(result);
        }
        for (target, toast_id) in self.timers.drain_expired_toasts(now) {
            let mut metadata = UiInputEventMetadata::new(now, UiInputSequence::new(0));
            metadata.synthetic = true;
            let mut result = input::dispatch_input_event(
                surface,
                &self.pointer,
                &self.navigation,
                UiInputEvent::ToastTimer(UiToastTimerInputEvent {
                    metadata,
                    target,
                    toast_id,
                }),
                None,
                None,
                self.diagnostics_mode,
            )?;
            self.record_text_service_lifecycle_from_result(surface, &mut result);
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

    fn record_text_service_lifecycle_from_result(
        &mut self,
        surface: &mut UiSurface,
        result: &mut UiInputDispatchResult,
    ) {
        finish_pending_text_focus_loss(
            &mut self.text_documents,
            &mut self.text_model_updates,
            surface,
        );
        append_ime_host_requests_for_result(result, &mut self.ime_host_requests);
        self.clipboard_host_requests.record_result(surface, result);
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
        self.dismiss_tooltip(surface);
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
        let routing = result.pointer_routing.as_ref();
        if let Some(routing) = routing {
            self.pointers
                .set_hovered_path_iter(pointer_id, routing.physical_bubble_route());
        } else {
            self.pointers.set_hovered_path(pointer_id, []);
        }
        match active_pointer_event.kind {
            UiPointerEventKind::Down => {
                self.pointers.press_button(
                    pointer_id,
                    active_pointer_event.button,
                    routing.and_then(|routing| routing.route_target),
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
            .or_else(|| routing.and_then(|routing| routing.capture_target));
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
            preedit_clauses: Vec::new(),
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
            | UiInputEvent::Clipboard(_)
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
        UiInputEvent::Clipboard(clipboard) => clipboard.metadata.timestamp,
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

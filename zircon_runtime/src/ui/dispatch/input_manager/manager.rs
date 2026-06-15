use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiInputDispatchResult, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiSubmenuHoverTimerInputEvent, UiToastTimerInputEvent,
        UiTypeaheadTimerInputEvent,
    },
    event_ui::UiNodeId,
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
        event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        let timestamp = input_event_timestamp(&event);
        let result = input::dispatch_input_event(surface, &self.pointer, &self.navigation, event)?;
        self.arm_timers_from_component_events(surface, timestamp, &result);
        Ok(result)
    }

    pub fn dispatch_window_event(
        &mut self,
        surface: &mut UiSurface,
        event: UiWindowInputPumpEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        input::dispatch_window_input_pump_event(surface, &self.pointer, &self.navigation, event)
    }

    pub fn dispatch_window_batch(
        &mut self,
        surface: &mut UiSurface,
        batch: UiWindowInputPumpBatch,
    ) -> Result<UiInputDispatchOutcome, UiTreeError> {
        let mut results = Vec::with_capacity(batch.events.len());
        for event in batch.events {
            results.push(self.dispatch_window_event(surface, event)?);
        }
        Ok(UiInputDispatchOutcome::from_results(surface, results))
    }

    pub fn tick(
        &mut self,
        surface: &mut UiSurface,
        now: UiInputTimestamp,
    ) -> Result<Vec<UiInputDispatchResult>, UiTreeError> {
        self.timers.record_tick(now);
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
        surface: &UiSurface,
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
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        tree::{UiTemplateNodeMetadata, UiTreeNode},
    };

    use crate::ui::surface::UiSurface;

    use super::UiInputManager;

    #[test]
    fn hovered_menu_option_arms_replaces_and_clears_submenu_hover_timer() {
        let target = UiNodeId::new(2);
        for component in ["MenuList", "ContextMenu", "DropdownPopup"] {
            let surface = submenu_hover_surface(component);
            let mut manager = UiInputManager::default();

            manager.arm_timers_from_component_events(
                &surface,
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
                &surface,
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
                &surface,
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
        let surface = toast_surface("surface-save", 4000);
        let mut manager = UiInputManager::default();

        manager.arm_timers_from_component_events(
            &surface,
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
            &surface,
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
            &surface,
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
            &surface,
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

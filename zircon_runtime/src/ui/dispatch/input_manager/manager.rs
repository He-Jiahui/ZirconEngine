use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiInputDispatchResult, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiSubmenuHoverTimerInputEvent, UiTypeaheadTimerInputEvent,
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
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        component::{UiComponentEvent, UiValue},
        dispatch::{
            UiComponentEventReport, UiDispatchReply, UiInputDispatchResult, UiInputEvent,
            UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiTextInputEvent,
        },
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        tree::{UiTemplateNodeMetadata, UiTreeNode},
    };

    use crate::ui::surface::UiSurface;

    use super::UiInputManager;

    #[test]
    fn hovered_menu_option_arms_replaces_and_clears_submenu_hover_timer() {
        let surface = submenu_hover_surface();
        let target = UiNodeId::new(2);
        let mut manager = UiInputManager::default();

        manager.arm_timers_from_component_events(
            &surface,
            UiInputTimestamp::from_micros(50),
            &hover_changed_result(target, "file"),
        );

        assert_eq!(
            manager.timers().submenu_hover_expiration(target),
            Some(UiInputTimestamp::from_micros(80_050))
        );
        assert_eq!(
            manager.timers().submenu_hover_option_id(target),
            Some("file")
        );

        manager.arm_timers_from_component_events(
            &surface,
            UiInputTimestamp::from_micros(70),
            &hover_changed_result(target, "edit"),
        );

        assert_eq!(
            manager.timers().submenu_hover_expiration(target),
            Some(UiInputTimestamp::from_micros(80_070))
        );
        assert_eq!(
            manager.timers().submenu_hover_option_id(target),
            Some("edit")
        );

        manager.arm_timers_from_component_events(
            &surface,
            UiInputTimestamp::from_micros(90),
            &hover_changed_result(target, ""),
        );

        assert_eq!(manager.timers().submenu_hover_expiration(target), None);
        assert_eq!(manager.timers().submenu_hover_option_id(target), None);
    }

    fn submenu_hover_surface() -> UiSurface {
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
            component: "MenuList".to_string(),
            control_id: Some("SceneMenu".to_string()),
            attributes: toml::from_str("submenu_hover_delay_ms = 80").unwrap(),
            ..Default::default()
        });
        surface.rebuild();
        surface
    }

    fn hover_changed_result(target: UiNodeId, option_id: &str) -> UiInputDispatchResult {
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
            event: UiComponentEvent::ValueChanged {
                property: "hovered_option_id".to_string(),
                value: UiValue::String(option_id.to_string()),
            },
            delivered: true,
            drag: None,
        });
        result
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
        UiInputEvent::Accessibility(accessibility) => accessibility.metadata.timestamp,
    }
}

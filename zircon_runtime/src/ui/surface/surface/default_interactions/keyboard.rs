use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiComponentKeyboardAction, UiValue},
    event_ui::UiNodeId,
    tree::UiTreeError,
    widget::UiWidgetBehavior,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};

use super::{widget_behavior, UiDefaultKeyboardActionReport};

impl UiSurface {
    pub(crate) fn apply_default_keyboard_component_action(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let Some(behavior) = self.default_keyboard_behavior(node_id)? else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };

        match behavior {
            UiWidgetBehavior::Button | UiWidgetBehavior::MenuItem => {
                let mut binding_reports = Vec::new();
                let event = UiComponentEvent::Commit {
                    property: "activated".to_string(),
                    value: UiValue::Bool(true),
                };
                let component_events = self.component_event_reports_for_bindings(
                    node_id,
                    UiEventKind::Click,
                    event,
                    false,
                )?;
                let component_events = if behavior == UiWidgetBehavior::MenuItem {
                    self.with_default_menu_item_popup_close_reports(
                        node_id,
                        component_events,
                        &mut binding_reports,
                    )?
                } else {
                    component_events
                };
                Ok(UiDefaultKeyboardActionReport {
                    handled: !component_events.is_empty(),
                    component_events,
                    binding_reports,
                })
            }
            UiWidgetBehavior::Toggle => {
                let Some(next_checked) = self.default_toggle_next_checked(node_id)? else {
                    return Ok(UiDefaultKeyboardActionReport::default());
                };
                let property = self.default_toggle_checked_property(node_id)?;
                let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
                    node_id,
                    property.clone(),
                    UiValue::Bool(next_checked),
                ))?;
                if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
                    return Ok(UiDefaultKeyboardActionReport::default());
                }
                let binding_reports = vec![report.binding];
                let event = UiComponentEvent::ValueChanged {
                    property,
                    value: UiValue::Bool(next_checked),
                };
                let component_events = self.component_event_reports_for_bindings(
                    node_id,
                    UiEventKind::Change,
                    event,
                    true,
                )?;
                Ok(UiDefaultKeyboardActionReport {
                    handled: true,
                    component_events,
                    binding_reports,
                })
            }
            UiWidgetBehavior::Radio => self.apply_default_radio_keyboard_action(node_id),
            UiWidgetBehavior::Disclosure => {
                let Some(next_expanded) = self.default_expanded_next(node_id)? else {
                    return Ok(UiDefaultKeyboardActionReport::default());
                };
                let property = self.default_open_property(node_id, "expanded")?;
                let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
                    node_id,
                    property,
                    UiValue::Bool(next_expanded),
                ))?;
                if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
                    return Ok(UiDefaultKeyboardActionReport::default());
                }
                let binding_reports = vec![report.binding];
                let event = UiComponentEvent::ToggleExpanded {
                    expanded: next_expanded,
                };
                let component_events = self.component_event_reports_for_bindings(
                    node_id,
                    UiEventKind::Toggle,
                    event,
                    true,
                )?;
                Ok(UiDefaultKeyboardActionReport {
                    handled: true,
                    component_events,
                    binding_reports,
                })
            }
            UiWidgetBehavior::Popup => self.apply_default_popup_keyboard_action(node_id),
            UiWidgetBehavior::Auto
            | UiWidgetBehavior::Passive
            | UiWidgetBehavior::RadioGroup
            | UiWidgetBehavior::Range
            | UiWidgetBehavior::Scrollbar
            | UiWidgetBehavior::ScrollbarThumb
            | UiWidgetBehavior::TextInput => Ok(UiDefaultKeyboardActionReport::default()),
        }
    }

    pub(crate) fn apply_default_semantic_keyboard_component_action(
        &mut self,
        node_id: UiNodeId,
        action: UiComponentKeyboardAction,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let Some(behavior) = self.default_keyboard_behavior(node_id)? else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };
        let action = semantic_keyboard_action_for_behavior(action, behavior);
        let event = UiComponentEvent::KeyboardAction { action };
        let mut component_events = Vec::new();
        for event_kind in semantic_keyboard_event_kinds(action) {
            component_events.extend(self.component_event_reports_for_bindings(
                node_id,
                *event_kind,
                event.clone(),
                true,
            )?);
        }
        Ok(UiDefaultKeyboardActionReport {
            handled: !component_events.is_empty(),
            component_events,
            binding_reports: Vec::new(),
        })
    }

    pub(crate) fn apply_default_semantic_keyboard_component_text(
        &mut self,
        node_id: UiNodeId,
        text: &str,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };
        if !self.widget_interaction_enabled(node_id, node, metadata) {
            return Ok(UiDefaultKeyboardActionReport::default());
        }

        let event = UiComponentEvent::KeyboardText {
            text: text.to_string(),
        };
        let component_events =
            self.component_event_reports_for_bindings(node_id, UiEventKind::Change, event, true)?;
        Ok(UiDefaultKeyboardActionReport {
            handled: !component_events.is_empty(),
            component_events,
            binding_reports: Vec::new(),
        })
    }

    fn default_keyboard_behavior(
        &self,
        node_id: UiNodeId,
    ) -> Result<Option<UiWidgetBehavior>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata) {
            return Ok(None);
        }
        Ok(Some(widget_behavior(metadata)))
    }
}

fn semantic_keyboard_action_for_behavior(
    action: UiComponentKeyboardAction,
    behavior: UiWidgetBehavior,
) -> UiComponentKeyboardAction {
    if !matches!(
        behavior,
        UiWidgetBehavior::Range | UiWidgetBehavior::Scrollbar | UiWidgetBehavior::ScrollbarThumb
    ) {
        return action;
    }

    match action {
        UiComponentKeyboardAction::Next => UiComponentKeyboardAction::Increment,
        UiComponentKeyboardAction::Previous => UiComponentKeyboardAction::Decrement,
        _ => action,
    }
}

fn semantic_keyboard_event_kinds(action: UiComponentKeyboardAction) -> &'static [UiEventKind] {
    match action {
        UiComponentKeyboardAction::Activate | UiComponentKeyboardAction::Cancel => &[
            UiEventKind::Click,
            UiEventKind::Change,
            UiEventKind::Toggle,
            UiEventKind::Submit,
        ],
        UiComponentKeyboardAction::Next
        | UiComponentKeyboardAction::Previous
        | UiComponentKeyboardAction::First
        | UiComponentKeyboardAction::Last
        | UiComponentKeyboardAction::Increment
        | UiComponentKeyboardAction::Decrement
        | UiComponentKeyboardAction::LargeIncrement
        | UiComponentKeyboardAction::LargeDecrement
        | UiComponentKeyboardAction::BeginEdit => &[UiEventKind::Change],
    }
}

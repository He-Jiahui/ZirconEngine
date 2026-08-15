use zircon_runtime_interface::ui::{
    binding::{UiBindingUpdateReport, UiEventKind},
    component::{UiComponentEvent, UiComponentEventKind, UiComponentState, UiValue},
    dispatch::{UiComponentEventReport, UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    surface::{UiPointerActivationPhase, UiPointerRoute},
    template::UiBindingRef,
    tree::{UiTemplateNodeMetadata, UiTreeError},
    widget::UiWidgetBehavior,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus};

use super::UiSurface;

mod keyboard;
mod popup;
mod radio;
mod range;
mod scrollbar;
mod table;
mod timers;
mod toast_timer;
mod tree_view;
mod tree_view_reparent;
mod tree_view_support;
mod tree_view_virtualization;

pub(super) use range::{range_navigation_action, UiDefaultRangeNavigationAction};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct UiDefaultRangePointerActionReport {
    pub handled_by: Option<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
    pub released_capture: Option<UiNodeId>,
    pub damage_node: Option<UiNodeId>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiDefaultKeyboardActionReport {
    pub handled: bool,
    pub component_events: Vec<UiComponentEventReport>,
    pub binding_reports: Vec<UiBindingUpdateReport>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::surface::surface) struct UiDefaultTreeViewPointerActionReport {
    pub handled_by: Option<UiNodeId>,
    pub damage_node: Option<UiNodeId>,
}

impl UiSurface {
    pub(super) fn apply_default_pointer_component_actions(
        &mut self,
        route: &UiPointerRoute,
        click_count: u8,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<(), UiTreeError> {
        if self.apply_default_tree_view_reorder_component_action(route, events, binding_reports)? {
            return Ok(());
        }
        if route.activation_phase == UiPointerActivationPhase::SecondaryRelease {
            let _ = self.apply_default_tree_view_rename_component_action(
                route,
                click_count,
                events,
                binding_reports,
            )?;
            return Ok(());
        }
        if route.activation_phase != UiPointerActivationPhase::PrimaryRelease {
            return Ok(());
        }
        if self.apply_default_popup_outside_dismissal_pointer(route, events, binding_reports)? {
            return Ok(());
        }
        let Some(node_id) = route.click_target else {
            return Ok(());
        };
        let Some(next_checked) = self.default_toggle_next_checked(node_id)? else {
            if self.apply_default_radio_component_action(node_id, events, binding_reports)? {
                return Ok(());
            }
            let tree_selection_handled =
                self.apply_default_tree_view_component_action(route, events, binding_reports)?;
            let tree_rename_handled = self.apply_default_tree_view_rename_component_action(
                route,
                click_count,
                events,
                binding_reports,
            )?;
            if tree_selection_handled || tree_rename_handled {
                return Ok(());
            }
            if self.apply_default_expanded_component_action(node_id, events, binding_reports)? {
                return Ok(());
            }
            if self.apply_default_popup_component_action(node_id, events, binding_reports)? {
                return Ok(());
            }
            if self.apply_default_button_component_action(
                node_id,
                click_count,
                events,
                binding_reports,
            )? {
                return Ok(());
            }
            return Ok(());
        };

        let property = self.default_toggle_checked_property(node_id)?;
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            node_id,
            property.clone(),
            UiValue::Bool(next_checked),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(());
        }
        binding_reports.push(report.binding);

        self.push_pointer_component_events(
            events,
            node_id,
            UiEventKind::Change,
            UiComponentEvent::ValueChanged {
                property,
                value: UiValue::Bool(next_checked),
            },
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(())
    }

    fn apply_default_button_component_action(
        &mut self,
        node_id: UiNodeId,
        click_count: u8,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(false);
        };
        if !matches!(
            widget_behavior(metadata),
            UiWidgetBehavior::Button | UiWidgetBehavior::MenuItem
        ) {
            return Ok(false);
        }
        if !self.widget_interaction_enabled(node_id, node, metadata) {
            return Ok(false);
        }

        self.push_pointer_component_events(
            events,
            node_id,
            UiEventKind::Click,
            UiComponentEvent::Commit {
                property: "activated".to_string(),
                value: UiValue::Bool(true),
            },
            UiPointerComponentEventReason::DefaultClick,
        )?;
        if click_count >= 2 {
            self.push_pointer_component_events(
                events,
                node_id,
                UiEventKind::DoubleClick,
                UiComponentEvent::Commit {
                    property: "double_activated".to_string(),
                    value: UiValue::Bool(true),
                },
                UiPointerComponentEventReason::DefaultDoubleClick,
            )?;
        }
        if widget_behavior(metadata) == UiWidgetBehavior::MenuItem {
            self.apply_default_menu_item_popup_close_pointer(node_id, events, binding_reports)?;
        }
        Ok(true)
    }

    fn apply_default_expanded_component_action(
        &mut self,
        node_id: UiNodeId,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(next_expanded) = self.default_expanded_next(node_id)? else {
            return Ok(false);
        };
        let property = self.default_open_property(node_id, "expanded")?;
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            node_id,
            property.clone(),
            UiValue::Bool(next_expanded),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(false);
        }
        binding_reports.push(report.binding);

        self.push_pointer_component_events(
            events,
            node_id,
            UiEventKind::Toggle,
            UiComponentEvent::ToggleExpanded {
                expanded: next_expanded,
            },
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(true)
    }

    fn default_toggle_next_checked(&self, node_id: UiNodeId) -> Result<Option<bool>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata)
            || !is_default_toggle_behavior(metadata)
        {
            return Ok(None);
        }
        let property = widget_checked_property(metadata);
        let current = self.default_toggle_current_checked(node_id, node, metadata, property);
        Ok(Some(!current))
    }

    fn default_toggle_current_checked(
        &self,
        node_id: UiNodeId,
        node: &zircon_runtime_interface::ui::tree::UiTreeNode,
        metadata: &UiTemplateNodeMetadata,
        property: &str,
    ) -> bool {
        let component_state = self.component_states.get(node_id);
        if property == "checked" {
            return bool_attribute_value(&metadata.attributes, property)
                .or_else(|| {
                    component_state.and_then(|state| bool_component_state_value(state, property))
                })
                .or_else(|| component_state.and_then(|state| state.flags.checked.then_some(true)))
                .unwrap_or(node.state_flags.checked || metadata.widget.checked.unwrap_or(false));
        }
        bool_attribute_value(&metadata.attributes, property)
            .or_else(|| {
                component_state.and_then(|state| bool_component_state_value(state, property))
            })
            .or(metadata.widget.checked)
            .unwrap_or(false)
    }

    fn default_expanded_next(&self, node_id: UiNodeId) -> Result<Option<bool>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata)
            || !is_default_expanded_behavior(metadata)
        {
            return Ok(None);
        }
        let property = widget_open_property(metadata, "expanded");
        let expanded = self.default_open_boolean_value(
            node_id,
            metadata,
            property,
            "expanded",
            &["expanded"],
            default_expanded_component_state(metadata),
        );
        Ok(Some(!expanded))
    }

    fn default_open_boolean_value(
        &self,
        node_id: UiNodeId,
        metadata: &UiTemplateNodeMetadata,
        property: &str,
        canonical_property: &str,
        fallback_properties: &[&str],
        default_value: bool,
    ) -> bool {
        let component_state = self.component_states.get(node_id);
        bool_attribute_value(&metadata.attributes, property)
            .or_else(|| {
                component_state.and_then(|state| bool_component_state_value(state, property))
            })
            .or_else(|| {
                fallback_properties
                    .iter()
                    .copied()
                    .filter(|fallback_property| *fallback_property != property)
                    .find_map(|fallback_property| {
                        bool_attribute_value(&metadata.attributes, fallback_property)
                    })
            })
            .or_else(|| {
                component_state.and_then(|state| {
                    fallback_properties
                        .iter()
                        .copied()
                        .filter(|fallback_property| *fallback_property != property)
                        .find_map(|fallback_property| {
                            bool_component_state_value(state, fallback_property)
                        })
                })
            })
            .or_else(|| {
                component_state
                    .and_then(|state| open_component_state_flag(state, canonical_property))
            })
            .unwrap_or(default_value)
    }

    pub(super) fn uses_typed_default_click_action(
        &self,
        node_id: UiNodeId,
    ) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(false);
        };
        Ok(is_default_button_behavior(metadata)
            || is_default_toggle_behavior(metadata)
            || is_default_radio_behavior(metadata)
            || is_default_expanded_behavior(metadata)
            || is_default_popup_behavior(metadata)
            || is_default_range_behavior(metadata)
            || is_default_scrollbar_behavior(metadata)
            || table::is_default_table_behavior(metadata)
            || tree_view::is_default_tree_view_behavior(metadata))
    }

    fn default_toggle_checked_property(&self, node_id: UiNodeId) -> Result<String, UiTreeError> {
        let metadata = self.template_metadata(node_id)?;
        Ok(widget_checked_property(metadata).to_string())
    }

    fn default_open_property(
        &self,
        node_id: UiNodeId,
        fallback: &'static str,
    ) -> Result<String, UiTreeError> {
        let metadata = self.template_metadata(node_id)?;
        Ok(widget_open_property(metadata, fallback).to_string())
    }

    fn template_metadata(&self, node_id: UiNodeId) -> Result<&UiTemplateNodeMetadata, UiTreeError> {
        self.tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?
            .template_metadata
            .as_ref()
            .ok_or(UiTreeError::MissingNode(node_id))
    }

    fn push_pointer_component_events_for_component_event_kind(
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
        let component_event_kind = event.kind();
        for binding in metadata.bindings.iter().filter(|binding| {
            binding.event == event_kind
                && binding_targets_component_event(binding, component_event_kind)
        }) {
            let mut component_event = UiPointerComponentEvent::new(
                &self.tree.tree_id,
                node_id,
                control_id,
                binding.id.as_str(),
                event_kind,
                event.clone(),
                reason,
            );
            if let Some(template_action) = self.template_action_for_binding(node_id, binding) {
                component_event = component_event.with_template_action(template_action);
            }
            events.push(component_event);
        }
        Ok(())
    }

    fn component_event_reports_for_bindings(
        &self,
        node_id: UiNodeId,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        require_component_event_token: bool,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(Vec::new());
        };
        let component_event_kind = event.kind();
        Ok(metadata
            .bindings
            .iter()
            .filter(|binding| {
                binding.event == event_kind
                    && (!require_component_event_token
                        || binding_targets_component_event(binding, component_event_kind))
            })
            .map(|_| UiComponentEventReport {
                target: node_id,
                event: event.clone(),
                delivered: true,
                drag: None,
                template_action: None,
            })
            .collect())
    }
}

fn is_default_toggle_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    widget_behavior(metadata) == UiWidgetBehavior::Toggle
}

fn is_default_button_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        widget_behavior(metadata),
        UiWidgetBehavior::Button | UiWidgetBehavior::MenuItem
    )
}

fn is_default_radio_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    widget_behavior(metadata) == UiWidgetBehavior::Radio
}

fn is_default_radio_group_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    widget_behavior(metadata) == UiWidgetBehavior::RadioGroup
}

fn is_default_expanded_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    widget_behavior(metadata) == UiWidgetBehavior::Disclosure
}

fn default_expanded_component_state(metadata: &UiTemplateNodeMetadata) -> bool {
    match metadata.widget.behavior {
        UiWidgetBehavior::Auto => matches!(
            metadata.component.as_str(),
            "Group" | "InspectorSection" | "TreeView"
        ),
        _ => matches!(
            metadata.component.as_str(),
            "Group" | "InspectorSection" | "TreeView"
        ),
    }
}

fn is_default_popup_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    widget_behavior(metadata) == UiWidgetBehavior::Popup
}

fn is_default_range_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    widget_behavior(metadata) == UiWidgetBehavior::Range
}

fn is_default_scrollbar_behavior(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        widget_behavior(metadata),
        UiWidgetBehavior::Scrollbar | UiWidgetBehavior::ScrollbarThumb
    )
}

fn widget_behavior(metadata: &UiTemplateNodeMetadata) -> UiWidgetBehavior {
    metadata.widget.resolved_behavior(&metadata.component)
}

fn widget_checked_property(metadata: &UiTemplateNodeMetadata) -> &str {
    metadata
        .widget
        .checked_property
        .as_deref()
        .unwrap_or("checked")
}

fn widget_open_property<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    fallback: &'static str,
) -> &'a str {
    metadata.widget.open_property.as_deref().unwrap_or(fallback)
}

fn bool_attribute_value(
    values: &std::collections::BTreeMap<String, toml::Value>,
    key: &str,
) -> Option<bool> {
    values.get(key).and_then(toml::Value::as_bool)
}

fn bool_component_state_value(state: &UiComponentState, property: &str) -> Option<bool> {
    match state.value(property) {
        Some(UiValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn open_component_state_flag(state: &UiComponentState, canonical_property: &str) -> Option<bool> {
    match canonical_property {
        "expanded" => state.flags.expanded.then_some(true),
        "popup_open" => state.flags.popup_open.then_some(true),
        _ => None,
    }
}

fn binding_targets_component_event(
    binding: &UiBindingRef,
    event_kind: UiComponentEventKind,
) -> bool {
    let Some(token) = component_event_kind_token(event_kind) else {
        return false;
    };
    binding.id.contains(token)
        || binding
            .route
            .as_deref()
            .is_some_and(|route| route.contains(token))
        || binding.action.as_ref().is_some_and(|action| {
            action
                .route
                .as_deref()
                .is_some_and(|route| route.contains(token))
                || action
                    .action
                    .as_deref()
                    .is_some_and(|action| action.contains(token))
        })
}

fn component_event_kind_token(event_kind: UiComponentEventKind) -> Option<&'static str> {
    match event_kind {
        UiComponentEventKind::ValueChanged => Some("ValueChanged"),
        UiComponentEventKind::Commit => Some("Commit"),
        UiComponentEventKind::KeyboardAction => Some("KeyboardAction"),
        UiComponentEventKind::KeyboardText => Some("KeyboardText"),
        UiComponentEventKind::TypeaheadExpired => Some("TypeaheadExpired"),
        UiComponentEventKind::Focus => Some("Focus"),
        UiComponentEventKind::Hover => Some("Hover"),
        UiComponentEventKind::Press => Some("Press"),
        UiComponentEventKind::BeginDrag => Some("BeginDrag"),
        UiComponentEventKind::DragDelta => Some("DragDelta"),
        UiComponentEventKind::LargeDragDelta => Some("LargeDragDelta"),
        UiComponentEventKind::EndDrag => Some("EndDrag"),
        UiComponentEventKind::DropHover => Some("DropHover"),
        UiComponentEventKind::ActiveDragTarget => Some("ActiveDragTarget"),
        UiComponentEventKind::OpenPopup => Some("OpenPopup"),
        UiComponentEventKind::OpenPopupAt => Some("OpenPopupAt"),
        UiComponentEventKind::ClosePopup => Some("ClosePopup"),
        UiComponentEventKind::SelectOption => Some("SelectOption"),
        UiComponentEventKind::ToggleExpanded => Some("ToggleExpanded"),
        UiComponentEventKind::AddElement => Some("AddElement"),
        UiComponentEventKind::SetElement => Some("SetElement"),
        UiComponentEventKind::RemoveElement => Some("RemoveElement"),
        UiComponentEventKind::MoveElement => Some("MoveElement"),
        UiComponentEventKind::AddMapEntry => Some("AddMapEntry"),
        UiComponentEventKind::SetMapEntry => Some("SetMapEntry"),
        UiComponentEventKind::RenameMapKey => Some("RenameMapKey"),
        UiComponentEventKind::RemoveMapEntry => Some("RemoveMapEntry"),
        UiComponentEventKind::DropReference => Some("DropReference"),
        UiComponentEventKind::ClearReference => Some("ClearReference"),
        UiComponentEventKind::LocateReference => Some("LocateReference"),
        UiComponentEventKind::OpenReference => Some("OpenReference"),
        UiComponentEventKind::SetVisibleRange => Some("SetVisibleRange"),
        UiComponentEventKind::SetPage => Some("SetPage"),
        UiComponentEventKind::SetWorldTransform => Some("SetWorldTransform"),
        UiComponentEventKind::SetWorldSurface => Some("SetWorldSurface"),
    }
}

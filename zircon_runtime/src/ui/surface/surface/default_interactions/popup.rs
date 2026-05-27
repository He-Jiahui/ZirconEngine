use zircon_runtime_interface::ui::{
    binding::{UiBindingUpdateReport, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{UiComponentEventReport, UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    surface::{UiArrangedNode, UiPointerRoute},
    tree::{UiTemplateNodeMetadata, UiTreeError},
    widget::UiWidgetBehavior,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use crate::ui::tree::UiRuntimeTreeAccessExt;

use super::{
    is_default_popup_behavior, widget_behavior, widget_open_property, UiDefaultKeyboardActionReport,
};

impl UiSurface {
    pub(super) fn apply_default_popup_component_action(
        &mut self,
        node_id: UiNodeId,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(next_popup_open) = self.default_popup_open_next(node_id)? else {
            return Ok(false);
        };
        let property = self.default_open_property(node_id, "popup_open")?;
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            node_id,
            property,
            UiValue::Bool(next_popup_open),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(false);
        }
        binding_reports.push(report.binding);

        let event = if next_popup_open {
            UiComponentEvent::OpenPopup
        } else {
            UiComponentEvent::ClosePopup
        };
        self.push_pointer_component_events_for_component_event_kind(
            events,
            node_id,
            UiEventKind::Click,
            event,
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(true)
    }

    pub(super) fn apply_default_popup_outside_dismissal_pointer(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<(), UiTreeError> {
        let Some(close) = self.default_popup_outside_close(route)? else {
            return Ok(());
        };
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            close.popup_id,
            close.property,
            UiValue::Bool(false),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(());
        }
        binding_reports.push(report.binding);
        self.push_pointer_component_events_for_component_event_kind(
            events,
            close.popup_id,
            UiEventKind::Click,
            UiComponentEvent::ClosePopup,
            UiPointerComponentEventReason::DefaultClick,
        )
    }

    pub(super) fn apply_default_popup_keyboard_action(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let Some(next_popup_open) = self.default_popup_open_next(node_id)? else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };
        let property = self.default_open_property(node_id, "popup_open")?;
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            node_id,
            property,
            UiValue::Bool(next_popup_open),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(UiDefaultKeyboardActionReport::default());
        }
        let binding_reports = vec![report.binding];
        let event = if next_popup_open {
            UiComponentEvent::OpenPopup
        } else {
            UiComponentEvent::ClosePopup
        };
        let component_events =
            self.component_event_reports_for_bindings(node_id, UiEventKind::Click, event, true)?;
        Ok(UiDefaultKeyboardActionReport {
            handled: true,
            component_events,
            binding_reports,
        })
    }

    pub(crate) fn apply_default_popup_dismissal_action(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let Some(close) = self.default_popup_ancestor_close(node_id)? else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            close.popup_id,
            close.property,
            UiValue::Bool(false),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(UiDefaultKeyboardActionReport::default());
        }
        let binding_reports = vec![report.binding];
        let component_events = self.component_event_reports_for_bindings(
            close.popup_id,
            UiEventKind::Click,
            UiComponentEvent::ClosePopup,
            true,
        )?;
        Ok(UiDefaultKeyboardActionReport {
            handled: true,
            component_events,
            binding_reports,
        })
    }

    pub(crate) fn default_popup_dismissal_target(
        &self,
        node_id: UiNodeId,
    ) -> Result<Option<(UiNodeId, String)>, UiTreeError> {
        Ok(self
            .default_popup_ancestor_close(node_id)?
            .map(|close| (close.popup_id, close.property)))
    }

    pub(super) fn apply_default_menu_item_popup_close_pointer(
        &mut self,
        menu_item_id: UiNodeId,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<(), UiTreeError> {
        let Some(close) = self.default_menu_item_popup_close(menu_item_id)? else {
            return Ok(());
        };
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            close.popup_id,
            close.property,
            UiValue::Bool(false),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(());
        }
        binding_reports.push(report.binding);
        self.push_pointer_component_events_for_component_event_kind(
            events,
            close.popup_id,
            UiEventKind::Click,
            UiComponentEvent::ClosePopup,
            UiPointerComponentEventReason::DefaultClick,
        )
    }

    pub(super) fn with_default_menu_item_popup_close_reports(
        &mut self,
        menu_item_id: UiNodeId,
        mut component_events: Vec<UiComponentEventReport>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        let Some(close) = self.default_menu_item_popup_close(menu_item_id)? else {
            return Ok(component_events);
        };
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            close.popup_id,
            close.property,
            UiValue::Bool(false),
        ))?;
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            binding_reports.push(report.binding);
            component_events.extend(self.component_event_reports_for_bindings(
                close.popup_id,
                UiEventKind::Click,
                UiComponentEvent::ClosePopup,
                true,
            )?);
        }
        Ok(component_events)
    }

    fn default_popup_open_next(&self, node_id: UiNodeId) -> Result<Option<bool>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata)
            || !is_default_popup_behavior(metadata)
        {
            return Ok(None);
        }
        let property = widget_open_property(metadata, "popup_open");
        let popup_open = self.default_open_boolean_value(
            node_id,
            metadata,
            property,
            "popup_open",
            &["popup_open", "open"],
            false,
        );
        Ok(Some(!popup_open))
    }

    fn default_menu_item_popup_close(
        &self,
        menu_item_id: UiNodeId,
    ) -> Result<Option<UiDefaultMenuPopupClose>, UiTreeError> {
        let menu_item = self
            .tree
            .node(menu_item_id)
            .ok_or(UiTreeError::MissingNode(menu_item_id))?;
        let Some(menu_item_metadata) = menu_item.template_metadata.as_ref() else {
            return Ok(None);
        };
        if widget_behavior(menu_item_metadata) != UiWidgetBehavior::MenuItem {
            return Ok(None);
        }

        self.default_popup_ancestor_close_from_parent(
            menu_item.parent,
            UiDefaultPopupCloseReason::MenuItemActivation,
        )
    }

    fn default_popup_ancestor_close(
        &self,
        node_id: UiNodeId,
    ) -> Result<Option<UiDefaultMenuPopupClose>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        self.default_popup_ancestor_close_from_parent(
            Some(node_id),
            UiDefaultPopupCloseReason::EscapeDismissal,
        )
        .and_then(|close| {
            if close.is_some() {
                Ok(close)
            } else {
                self.default_popup_ancestor_close_from_parent(
                    node.parent,
                    UiDefaultPopupCloseReason::EscapeDismissal,
                )
            }
        })
    }

    fn default_popup_ancestor_close_from_parent(
        &self,
        mut current: Option<UiNodeId>,
        reason: UiDefaultPopupCloseReason,
    ) -> Result<Option<UiDefaultMenuPopupClose>, UiTreeError> {
        while let Some(node_id) = current {
            let node = self
                .tree
                .node(node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if let Some(metadata) = node.template_metadata.as_ref() {
                if is_default_popup_behavior(metadata)
                    && self.widget_interaction_enabled(node_id, node, metadata)
                {
                    let property = widget_open_property(metadata, "popup_open");
                    let popup_open = self.default_open_boolean_value(
                        node_id,
                        metadata,
                        property,
                        "popup_open",
                        &["popup_open", "open"],
                        false,
                    );
                    if popup_open {
                        if reason == UiDefaultPopupCloseReason::EscapeDismissal
                            && !popup_escape_dismiss_enabled(metadata)
                        {
                            return Ok(None);
                        }
                        return Ok(Some(UiDefaultMenuPopupClose {
                            popup_id: node_id,
                            property: property.to_string(),
                        }));
                    }
                }
            }
            current = node.parent;
        }
        Ok(None)
    }

    fn default_popup_outside_close(
        &self,
        route: &UiPointerRoute,
    ) -> Result<Option<UiDefaultMenuPopupClose>, UiTreeError> {
        for arranged in self.arranged_tree.nodes.iter().rev() {
            let node_id = arranged.node_id;
            let node = self
                .tree
                .node(node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            let Some(metadata) = node.template_metadata.as_ref() else {
                continue;
            };
            if !is_default_popup_behavior(metadata)
                || !self.widget_interaction_enabled(node_id, node, metadata)
            {
                continue;
            }
            let property = widget_open_property(metadata, "popup_open");
            let popup_open = self.default_open_boolean_value(
                node_id,
                metadata,
                property,
                "popup_open",
                &["popup_open", "open"],
                false,
            );
            if !popup_open || pointer_route_is_inside_popup(route, arranged) {
                continue;
            }
            if !popup_backdrop_click_enabled(metadata) {
                return Ok(None);
            }
            return Ok(Some(UiDefaultMenuPopupClose {
                popup_id: node_id,
                property: property.to_string(),
            }));
        }
        Ok(None)
    }
}

struct UiDefaultMenuPopupClose {
    popup_id: UiNodeId,
    property: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiDefaultPopupCloseReason {
    MenuItemActivation,
    EscapeDismissal,
}

fn popup_escape_dismiss_enabled(metadata: &UiTemplateNodeMetadata) -> bool {
    !bool_attribute(metadata, "disable_escape_key_down")
        && !bool_attribute(metadata, "disable_escape_dismiss")
}

fn popup_backdrop_click_enabled(metadata: &UiTemplateNodeMetadata) -> bool {
    !matches!(
        metadata
            .attributes
            .get("close_on_backdrop_click")
            .and_then(toml::Value::as_bool),
        Some(false)
    ) && !bool_attribute(metadata, "disable_backdrop_click")
        && !bool_attribute(metadata, "disable_outside_dismiss")
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> bool {
    metadata
        .attributes
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn pointer_route_is_inside_popup(route: &UiPointerRoute, popup: &UiArrangedNode) -> bool {
    route.bubbled.contains(&popup.node_id)
        || route.stacked.contains(&popup.node_id)
        || popup
            .frame
            .intersection(popup.clip_frame)
            .unwrap_or(popup.frame)
            .contains_point(route.point)
}

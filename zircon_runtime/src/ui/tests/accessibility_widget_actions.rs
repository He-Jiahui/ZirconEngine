use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityContract,
        UiAccessibilityDiagnosticCode,
    },
    binding::{UiBindingSourceKind, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiDispatchEffect,
        UiDispatchHostRequestKind, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata,
        UiTooltipEffectKind,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

mod disclosure_actions;
mod popup_actions;
mod tooltip_menu;

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}

fn state(clickable: bool, focusable: bool) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable,
        hoverable: clickable,
        focusable,
        ..UiStateFlags::default()
    }
}

fn root_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.accessibility.widget_actions"));
    surface.tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 120.0)),
    );
    surface
}

fn dispatch_accessibility(
    surface: &mut UiSurface,
    target: UiNodeId,
    action: UiAccessibilityAction,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Accessibility(UiAccessibilityInputEvent {
                metadata: UiInputEventMetadata::default(),
                request: UiAccessibilityActionRequest {
                    target,
                    action,
                    ..UiAccessibilityActionRequest::default()
                },
            }),
        )
        .unwrap()
}

fn assert_accessibility_binding_report(
    result: &UiInputDispatchResult,
    expected_applied_count: u64,
) {
    assert_eq!(result.binding_reports.len(), 1);
    let report = &result.binding_reports[0];
    assert_eq!(report.applied_count, expected_applied_count);
    assert_eq!(report.rejected_count, 0);
    assert_eq!(
        report.updates.first().map(|update| update.source.kind),
        Some(UiBindingSourceKind::AccessibilityAction)
    );
}

fn assert_widget_binding_report(result: &UiInputDispatchResult) {
    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(
        result
            .binding_reports
            .first()
            .and_then(|report| report.updates.first())
            .map(|update| update.source.kind),
        Some(UiBindingSourceKind::WidgetBehavior)
    );
}

fn insert_runtime_open_widget(
    surface: &mut UiSurface,
    component: &str,
    behavior: UiWidgetBehavior,
    open_property: &str,
) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new(format!("root/{component}")))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some(component.to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior,
                        open_property: Some(open_property.to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_runtime_popup_dialog(
    surface: &mut UiSurface,
    open_property: &str,
    actions: Vec<UiAccessibilityAction>,
) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/RuntimePopupDialog"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimePopupDialog".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Dialog,
                        name: Some("RuntimePopupDialog".to_string()),
                        actions,
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some(open_property.to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_runtime_popup_menu(surface: &mut UiSurface, open_property: &str) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/RuntimePopupMenu"))
                .with_frame(UiFrame::new(4.0, 4.0, 120.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimePopupMenu".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Menu,
                        name: Some("RuntimePopupMenu".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some(open_property.to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_runtime_tooltip(surface: &mut UiSurface) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/RuntimeTooltip"))
                .with_frame(UiFrame::new(8.0, 8.0, 100.0, 20.0))
                .with_state_flags(state(false, false))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Tooltip".to_string(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::Tooltip,
                        name: Some("Runtime tooltip".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn insert_runtime_menu_item_in_popup_without_item_binding(surface: &mut UiSurface) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/MenuPopup"))
                .with_frame(UiFrame::new(4.0, 4.0, 140.0, 72.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    bindings: vec![binding("MenuPopup/ClosePopup", UiEventKind::Click)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/MenuPopup/Item"))
                .with_frame(UiFrame::new(12.0, 12.0, 100.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(state(true, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeMenuItem".to_string(),
                    a11y: UiAccessibilityContract {
                        name: Some("Runtime menu item".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::MenuItem,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

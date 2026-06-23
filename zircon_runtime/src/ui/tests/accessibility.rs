use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yCheckedState, UiA11yRole, UiA11yTextSelection, UiAccessibilityAction,
        UiAccessibilityActionRequest, UiAccessibilityContract, UiAccessibilityDiagnosticCode,
    },
    binding::UiBindingSourceKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    tree::{UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}

fn metadata(component: &str, attributes: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        attributes: toml::from_str(attributes).unwrap(),
        ..UiTemplateNodeMetadata::default()
    }
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
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.accessibility"));
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
    dispatch_accessibility_with_value(surface, target, action, None, None)
}

fn dispatch_accessibility_with_value(
    surface: &mut UiSurface,
    target: UiNodeId,
    action: UiAccessibilityAction,
    value: Option<&str>,
    numeric_value: Option<f64>,
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
                    value: value.map(str::to_string),
                    numeric_value,
                    ..UiAccessibilityActionRequest::default()
                },
            }),
        )
        .unwrap()
}

fn has_note(result: &UiInputDispatchResult, needle: &str) -> bool {
    result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.contains(needle))
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
    let report = &result.binding_reports[0];
    assert!(report.applied_count > 0);
    assert_eq!(report.rejected_count, 0);
    assert_eq!(
        report.updates.first().map(|update| update.source.kind),
        Some(UiBindingSourceKind::WidgetBehavior)
    );
}

mod activation_actions;
mod description_references;
mod extraction;
mod focus_diagnostics;
mod naming_relations;
mod value_actions;

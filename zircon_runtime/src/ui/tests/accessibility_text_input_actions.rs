use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiA11yTextSelection, UiAccessibilityAction, UiAccessibilityActionRequest,
        UiAccessibilityContract,
    },
    binding::UiBindingSourceKind,
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

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
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.accessibility.text_input_actions"));
    surface.tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 120.0)),
    );
    surface
}

fn insert_text_input(surface: &mut UiSurface, attributes: &str) {
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/text-input"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::TextInput,
                        actions: vec![
                            UiAccessibilityAction::Focus,
                            UiAccessibilityAction::SetValue,
                            UiAccessibilityAction::ReplaceSelectedText,
                            UiAccessibilityAction::SetTextSelection,
                        ],
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn dispatch_set_value(surface: &mut UiSurface, value: &str) -> UiInputDispatchResult {
    dispatch_text_action(surface, UiAccessibilityAction::SetValue, value)
}

fn dispatch_replace_selected_text(surface: &mut UiSurface, value: &str) -> UiInputDispatchResult {
    dispatch_text_action(surface, UiAccessibilityAction::ReplaceSelectedText, value)
}

fn dispatch_set_text_selection(
    surface: &mut UiSurface,
    selection: UiA11yTextSelection,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Accessibility(UiAccessibilityInputEvent {
                metadata: UiInputEventMetadata::default(),
                request: UiAccessibilityActionRequest {
                    target: id(2),
                    action: UiAccessibilityAction::SetTextSelection,
                    text_selection: Some(selection),
                    ..UiAccessibilityActionRequest::default()
                },
            }),
        )
        .unwrap()
}

fn dispatch_text_action(
    surface: &mut UiSurface,
    action: UiAccessibilityAction,
    value: &str,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Accessibility(UiAccessibilityInputEvent {
                metadata: UiInputEventMetadata::default(),
                request: UiAccessibilityActionRequest {
                    target: id(2),
                    action,
                    value: Some(value.to_string()),
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

#[test]
fn accessibility_set_value_rejects_read_only_text_input() {
    let mut surface = root_surface();
    insert_text_input(&mut surface, "text = 'Old value'\nread_only = true");
    surface.rebuild();

    let result = dispatch_set_value(&mut surface, "New value");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(has_note(&result, "status=rejected"));
    assert!(has_note(&result, "code=read_only"));
    assert!(result.binding_reports.is_empty());
    assert!(result.component_events.is_empty());
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("Old value"));
}

#[test]
fn accessibility_set_value_applies_text_input_constraints_before_mutation() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = '0'\ninput_filter = 'digits'\nmax_chars = 3\nmultiline = false\ncaret_offset = 1\nselection_anchor = 0\nselection_focus = 1",
    );
    surface.rebuild();

    let result = dispatch_set_value(&mut surface, "a1\n23b4");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(has_note(&result, "accessibility_text_value_sanitized"));
    assert_eq!(result.binding_reports.len(), 8);
    assert_eq!(
        result
            .binding_reports
            .iter()
            .map(|report| report.applied_count)
            .sum::<u64>(),
        16
    );
    assert!(result
        .binding_reports
        .iter()
        .all(|report| report.rejected_count == 0
            && report.updates.first().map(|update| update.source.kind)
                == Some(UiBindingSourceKind::AccessibilityAction)));

    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("123"));
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(3));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(3)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(3));
    assert_eq!(
        metadata.attributes["composition_start"].as_integer(),
        Some(3)
    );
    assert_eq!(metadata.attributes["composition_end"].as_integer(), Some(3));
    assert_eq!(metadata.attributes["composition_text"].as_str(), Some(""));
    assert_eq!(
        metadata.attributes["composition_restore_text"].as_str(),
        Some("")
    );

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("constrained text input remains exposed");
    assert_eq!(
        node.state.text_selection,
        Some(UiA11yTextSelection::collapsed(3))
    );
    assert_eq!(node.state.value.as_deref(), Some("123"));
}

#[test]
fn accessibility_set_value_clears_active_composition_metadata() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'abcd'\ncaret_offset = 3\nselection_anchor = 3\nselection_focus = 3\ncomposition_start = 1\ncomposition_end = 3\ncomposition_text = 'XY'\ncomposition_restore_text = 'bc'",
    );
    surface.rebuild();

    let result = dispatch_set_value(&mut surface, "Hello");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.binding_reports.len(), 8);
    assert_eq!(
        result
            .binding_reports
            .iter()
            .map(|report| report.applied_count)
            .sum::<u64>(),
        16
    );
    assert!(has_note(
        &result,
        "accessibility_text_composition_changed:composition_start"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_composition_changed:composition_end"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_composition_changed:composition_text"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_composition_changed:composition_restore_text"
    ));

    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("Hello"));
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(5));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(5)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(5));
    assert_eq!(
        metadata.attributes["composition_start"].as_integer(),
        Some(5)
    );
    assert_eq!(metadata.attributes["composition_end"].as_integer(), Some(5));
    assert_eq!(metadata.attributes["composition_text"].as_str(), Some(""));
    assert_eq!(
        metadata.attributes["composition_restore_text"].as_str(),
        Some("")
    );

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("composition-cleared text input remains exposed");
    assert_eq!(
        node.state.text_selection,
        Some(UiA11yTextSelection::collapsed(5))
    );
    assert_eq!(node.state.value.as_deref(), Some("Hello"));
}

#[test]
fn accessibility_replace_selected_text_updates_selected_range_only() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'abcd'\ncaret_offset = 3\nselection_anchor = 1\nselection_focus = 3\ncomposition_start = 1\ncomposition_end = 3\ncomposition_text = 'XY'\ncomposition_restore_text = 'bc'",
    );
    surface.rebuild();

    let result = dispatch_replace_selected_text(&mut surface, "Z");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.replace_selected_text")
    );
    assert_eq!(result.binding_reports.len(), 8);
    assert_eq!(
        result
            .binding_reports
            .iter()
            .map(|report| report.applied_count)
            .sum::<u64>(),
        16
    );
    assert!(has_note(
        &result,
        "accessibility_text_selection_changed:caret_offset"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_composition_changed:composition_text"
    ));

    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("aZd"));
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(2));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(2)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(2));
    assert_eq!(
        metadata.attributes["composition_start"].as_integer(),
        Some(2)
    );
    assert_eq!(metadata.attributes["composition_end"].as_integer(), Some(2));
    assert_eq!(metadata.attributes["composition_text"].as_str(), Some(""));
    assert_eq!(
        metadata.attributes["composition_restore_text"].as_str(),
        Some("")
    );

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("replace-selected text input remains exposed");
    assert_eq!(
        node.state.text_selection,
        Some(UiA11yTextSelection::collapsed(2))
    );
    assert_eq!(node.state.value.as_deref(), Some("aZd"));
}

#[test]
fn accessibility_replace_selected_text_applies_constraints_to_selected_range() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'a0d'\ninput_filter = 'digits'\nmax_chars = 3\ncaret_offset = 2\nselection_anchor = 1\nselection_focus = 2",
    );
    surface.rebuild();

    let result = dispatch_replace_selected_text(&mut surface, "x123y");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(has_note(
        &result,
        "accessibility_replace_selected_text_sanitized"
    ));
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("a1d"));
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(2));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(2)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(2));
    assert_eq!(
        metadata.attributes["composition_start"].as_integer(),
        Some(2)
    );
    assert_eq!(metadata.attributes["composition_end"].as_integer(), Some(2));
}

#[test]
fn accessibility_set_text_selection_updates_read_only_text_input_selection() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'abcdef'\nread_only = true\ncaret_offset = 0\nselection_anchor = 0\nselection_focus = 0\ncomposition_start = 1\ncomposition_end = 3\ncomposition_text = 'bc'\ncomposition_restore_text = 'bc'",
    );
    surface.rebuild();

    let result = dispatch_set_text_selection(
        &mut surface,
        UiA11yTextSelection {
            caret: 4,
            anchor: 1,
            focus: 4,
        },
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.set_text_selection")
    );
    assert!(result.component_events.is_empty());
    assert_eq!(result.binding_reports.len(), 7);
    assert_eq!(
        result
            .binding_reports
            .iter()
            .map(|report| report.applied_count)
            .sum::<u64>(),
        14
    );
    assert!(has_note(
        &result,
        "accessibility_text_selection_changed:selection_anchor"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_composition_changed:composition_text"
    ));

    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("abcdef"));
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(4));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(1)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(4));
    assert_eq!(
        metadata.attributes["composition_start"].as_integer(),
        Some(4)
    );
    assert_eq!(metadata.attributes["composition_end"].as_integer(), Some(4));
    assert_eq!(metadata.attributes["composition_text"].as_str(), Some(""));
    assert_eq!(
        metadata.attributes["composition_restore_text"].as_str(),
        Some("")
    );

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("selection-updated text input remains exposed");
    assert_eq!(
        node.state.text_selection,
        Some(UiA11yTextSelection {
            caret: 4,
            anchor: 1,
            focus: 4,
        })
    );
    assert_eq!(node.state.value.as_deref(), Some("abcdef"));
}

#[test]
fn accessibility_set_text_selection_preserves_distinct_clamped_caret_offset() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = \"a\\u00E9z\"\ncaret_offset = 0\nselection_anchor = 0\nselection_focus = 0\ncomposition_start = 3\ncomposition_end = 4\ncomposition_text = 'z'\ncomposition_restore_text = 'z'",
    );
    surface.rebuild();

    let result = dispatch_set_text_selection(
        &mut surface,
        UiA11yTextSelection {
            caret: 2,
            anchor: 0,
            focus: 4,
        },
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(1));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(0)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(4));
    assert_eq!(
        metadata.attributes["composition_start"].as_integer(),
        Some(1)
    );
    assert_eq!(metadata.attributes["composition_end"].as_integer(), Some(1));
    assert_eq!(metadata.attributes["composition_text"].as_str(), Some(""));
    assert_eq!(
        metadata.attributes["composition_restore_text"].as_str(),
        Some("")
    );

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("distinct-caret text input remains exposed");
    assert_eq!(
        node.state.text_selection,
        Some(UiA11yTextSelection {
            caret: 1,
            anchor: 0,
            focus: 4,
        })
    );
}

#[test]
fn accessibility_set_text_selection_clamps_invalid_utf8_offsets() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = \"a\\u00E9z\"\ncaret_offset = 0\nselection_anchor = 0\nselection_focus = 0",
    );
    surface.rebuild();

    let result = dispatch_set_text_selection(
        &mut surface,
        UiA11yTextSelection {
            caret: 99,
            anchor: 2,
            focus: 99,
        },
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(4));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(1)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(4));

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(2))
        .expect("utf8-clamped text input remains exposed");
    assert_eq!(
        node.state.text_selection,
        Some(UiA11yTextSelection {
            caret: 4,
            anchor: 1,
            focus: 4,
        })
    );
}

use crate::ui::{
    dispatch::{UiInputManager, UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiA11yTextSelection, UiAccessibilityAction, UiAccessibilityActionRequest,
        UiAccessibilityContract,
    },
    binding::UiBindingSourceKind,
    component::UiComponentEvent,
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    text::UiTextEditSource,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetContract, UiWidgetEvent},
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

fn dispatch_text_action_with_manager(
    manager: &mut UiInputManager,
    surface: &mut UiSurface,
    action: UiAccessibilityAction,
    value: &str,
) -> UiInputDispatchResult {
    manager
        .dispatch_input_event(
            surface,
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

fn text_layout_revision(surface: &UiSurface) -> u64 {
    surface
        .tree
        .node(id(2))
        .and_then(|node| node.layout_cache.retained_text_layout_revision())
        .expect("text layout revision must remain reusable")
}

fn assert_accessibility_text_transaction(
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
fn accessibility_set_value_rejects_reserved_value_property_without_partial_text_state() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/reserved-value-property"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    attributes: toml::from_str(
                        "visibility = 'Old value'\ncaret_offset = 3\nselection_anchor = 1\nselection_focus = 3",
                    )
                    .unwrap(),
                    a11y: UiAccessibilityContract {
                        role: UiA11yRole::TextInput,
                        actions: vec![UiAccessibilityAction::SetValue],
                        ..UiAccessibilityContract::default()
                    },
                    widget: UiWidgetContract {
                        value_property: Some("visibility".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let result = dispatch_set_value(&mut surface, "New value");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(has_note(&result, "status=rejected"));
    assert!(has_note(&result, "code=reserved_value_property"));
    assert!(result.binding_reports.is_empty());
    assert!(result.component_events.is_empty());
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(
        metadata.attributes["visibility"].as_str(),
        Some("Old value")
    );
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(3));
    assert_eq!(
        metadata.attributes["selection_anchor"].as_integer(),
        Some(1)
    );
    assert_eq!(metadata.attributes["selection_focus"].as_integer(), Some(3));
    assert!(!metadata.attributes.contains_key("composition_start"));
}

#[test]
fn accessibility_set_value_applies_text_input_constraints_before_mutation() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = '0'\ninput_filter = 'digits'\nmax_chars = 3\nmultiline = false\ncaret_offset = 1\nselection_anchor = 0\nselection_focus = 1",
    );
    surface.rebuild();
    surface.clear_dirty_flags();
    let revision_before = text_layout_revision(&surface);

    let result = dispatch_set_value(&mut surface, "a1\n23b4");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(has_note(&result, "accessibility_text_value_sanitized"));
    let receipt = result
        .diagnostics
        .text_constraint
        .expect("accessibility set value publishes the shared typed constraint receipt");
    assert_eq!(receipt.removed_hard_line_count, 1);
    assert_eq!(receipt.removed_filter_scalar_count, 2);
    assert!(receipt.max_graphemes_truncated);
    assert_accessibility_text_transaction(&result, 20);
    assert_eq!(text_layout_revision(&surface), revision_before + 1);

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
fn accessibility_set_value_updates_secure_text_without_exposing_its_value_or_selection() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'Old secret'\nsecure = true\ncaret_offset = 10\nselection_anchor = 0\nselection_focus = 10",
    );
    surface.rebuild();

    let result = dispatch_set_value(&mut surface, "New secret");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(result.diagnostics.secure_text_redacted);
    let reference = match &result.component_events.as_slice() {
        [report] => match &report.event {
            UiComponentEvent::SecureValueChanged { reference, .. } => reference,
            event => panic!("expected secure value event, got {event:?}"),
        },
        reports => panic!("expected one secure value report, got {reports:?}"),
    };
    assert_eq!(
        surface.resolve_secure_text_value(reference),
        Some("New secret")
    );
    let encoded = serde_json::to_string(&result).unwrap();
    assert!(!encoded.contains("Old secret"));
    assert!(!encoded.contains("New secret"));
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("New secret"));

    let snapshot_node = surface
        .accessibility_snapshot()
        .node(id(2))
        .expect("secure text input remains discoverable");
    assert_eq!(snapshot_node.name, None);
    assert_eq!(snapshot_node.state.value, None);
    assert_eq!(snapshot_node.state.text_selection, None);
    assert!(
        snapshot_node
            .actions
            .contains(&UiAccessibilityAction::SetValue)
    );
    assert!(
        !snapshot_node
            .actions
            .contains(&UiAccessibilityAction::ReplaceSelectedText)
    );
    assert!(
        !snapshot_node
            .actions
            .contains(&UiAccessibilityAction::SetTextSelection)
    );
}

#[test]
fn accessibility_rejects_forged_secure_text_selection_request() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'Old secret'\nsecure = true\ncaret_offset = 0\nselection_anchor = 0\nselection_focus = 0",
    );
    surface.rebuild();

    let result = dispatch_set_text_selection(
        &mut surface,
        UiA11yTextSelection {
            caret: 3,
            anchor: 0,
            focus: 3,
        },
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(has_note(
        &result,
        "target does not expose set text selection action"
    ));
    let metadata = surface
        .tree
        .node(id(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["text"].as_str(), Some("Old secret"));
    assert_eq!(metadata.attributes["caret_offset"].as_integer(), Some(0));
}

#[test]
fn accessibility_set_value_clears_active_composition_metadata() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'abcd'\ncaret_offset = 3\nselection_anchor = 3\nselection_focus = 3\ncomposition_start = 1\ncomposition_end = 3\ncomposition_text = 'XY'\ncomposition_restore_text = 'bc'\ncomposition_clauses = [{ start_byte = 0, end_byte = 2, kind = 'input' }]",
    );
    surface.rebuild();

    let result = dispatch_set_value(&mut surface, "Hello");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_accessibility_text_transaction(&result, 20);
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_start"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_end"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_text"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_restore_text"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_clauses"
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
    assert_eq!(
        metadata.attributes["composition_clauses"].as_array(),
        Some([].as_slice())
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
        "text = 'abcd'\ncaret_offset = 3\nselection_anchor = 1\nselection_focus = 3\ncomposition_start = 1\ncomposition_end = 3\ncomposition_text = 'XY'\ncomposition_restore_text = 'bc'\ncomposition_clauses = [{ start_byte = 0, end_byte = 2, kind = 'target_converted' }]",
    );
    surface.rebuild();

    let result = dispatch_replace_selected_text(&mut surface, "Z");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.replace_selected_text")
    );
    assert_accessibility_text_transaction(&result, 20);
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:caret_offset"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_text"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_clauses"
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
    assert_eq!(
        metadata.attributes["composition_clauses"].as_array(),
        Some([].as_slice())
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
fn input_manager_accessibility_replacements_share_one_retained_document() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'abcd'\ncaret_offset = 3\nselection_anchor = 1\nselection_focus = 3",
    );
    surface.rebuild();
    let mut manager = UiInputManager::default();

    let first = dispatch_text_action_with_manager(
        &mut manager,
        &mut surface,
        UiAccessibilityAction::ReplaceSelectedText,
        "Z",
    );
    let second = dispatch_text_action_with_manager(
        &mut manager,
        &mut surface,
        UiAccessibilityAction::ReplaceSelectedText,
        "Q",
    );

    let UiWidgetEvent::TextEditChange { receipt: first } = &first.widget_events[0] else {
        panic!("expected first accessibility text edit receipt");
    };
    let UiWidgetEvent::TextEditChange { receipt: second } = &second.widget_events[0] else {
        panic!("expected second accessibility text edit receipt");
    };
    assert_eq!(first.document_id, second.document_id);
    assert_eq!(first.previous_revision.get(), 0);
    assert_eq!(first.revision.get(), 1);
    assert_eq!(second.previous_revision.get(), 1);
    assert_eq!(second.revision.get(), 2);
    assert_eq!(first.source, UiTextEditSource::Accessibility);
    assert_eq!(second.source, UiTextEditSource::Accessibility);
    assert_eq!(first.changed.old.start_byte, 1);
    assert_eq!(first.changed.old.end_byte, 3);
    assert_eq!(second.changed.old.start_byte, 2);
    assert_eq!(second.changed.old.end_byte, 2);
    let metadata = surface
        .tree
        .node(id(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("text input metadata");
    assert_eq!(metadata.attributes["text"].as_str(), Some("aZQd"));
}

#[test]
fn input_manager_accessibility_set_value_and_replace_share_document_revision_chain() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'abcd'\ncaret_offset = 4\nselection_anchor = 4\nselection_focus = 4",
    );
    surface.rebuild();
    let mut manager = UiInputManager::default();

    let set_value = dispatch_text_action_with_manager(
        &mut manager,
        &mut surface,
        UiAccessibilityAction::SetValue,
        "xy",
    );
    let replace = dispatch_text_action_with_manager(
        &mut manager,
        &mut surface,
        UiAccessibilityAction::ReplaceSelectedText,
        "Z",
    );

    let UiWidgetEvent::TextEditChange {
        receipt: set_value,
    } = &set_value.widget_events[0]
    else {
        panic!("expected accessibility set-value receipt");
    };
    let UiWidgetEvent::TextEditChange { receipt: replace } = &replace.widget_events[0] else {
        panic!("expected accessibility replacement receipt");
    };
    assert_eq!(set_value.document_id, replace.document_id);
    assert_eq!(set_value.previous_revision.get(), 0);
    assert_eq!(set_value.revision.get(), 1);
    assert_eq!(replace.previous_revision.get(), 1);
    assert_eq!(replace.revision.get(), 2);
    assert_eq!(set_value.source, UiTextEditSource::Accessibility);
    assert_eq!(set_value.changed.old.start_byte, 0);
    assert_eq!(set_value.changed.old.end_byte, 4);
    assert_eq!(set_value.changed.new.start_byte, 0);
    assert_eq!(set_value.changed.new.end_byte, 2);
    let metadata = surface
        .tree
        .node(id(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("text input metadata");
    assert_eq!(metadata.attributes["text"].as_str(), Some("xyZ"));
}

#[test]
fn accessibility_replace_selected_text_applies_constraints_to_selected_range() {
    let mut surface = root_surface();
    insert_text_input(
        &mut surface,
        "text = 'a0d'\ninput_filter = 'digits'\nmax_chars = 3\ncaret_offset = 2\nselection_anchor = 1\nselection_focus = 2",
    );
    surface.rebuild();
    surface.clear_dirty_flags();
    let revision_before = text_layout_revision(&surface);

    let result = dispatch_replace_selected_text(&mut surface, "x123y");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(has_note(
        &result,
        "accessibility_replace_selected_text_sanitized"
    ));
    let receipt = result
        .diagnostics
        .text_constraint
        .expect("accessibility replacement publishes the shared typed constraint receipt");
    assert_eq!(receipt.removed_hard_line_count, 0);
    assert_eq!(receipt.removed_filter_scalar_count, 2);
    assert!(receipt.max_graphemes_truncated);
    assert_accessibility_text_transaction(&result, 16);
    assert_eq!(text_layout_revision(&surface), revision_before + 1);
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
        "text = 'abcdef'\nread_only = true\ncaret_offset = 0\ncaret_affinity = 'upstream'\nselection_anchor = 0\nselection_focus = 0\ncomposition_start = 1\ncomposition_end = 3\ncomposition_text = 'bc'\ncomposition_restore_text = 'bc'",
    );
    surface.rebuild();
    surface.clear_dirty_flags();
    let revision_before = text_layout_revision(&surface);

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
    assert_accessibility_text_transaction(&result, 18);
    assert_eq!(text_layout_revision(&surface), revision_before);
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:selection_anchor"
    ));
    assert!(has_note(
        &result,
        "accessibility_text_state_changed:composition_text"
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
        metadata.attributes["caret_affinity"].as_str(),
        Some("downstream")
    );
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

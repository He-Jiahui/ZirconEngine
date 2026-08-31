use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState, UiNumberInputCommitStatus,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    text::{
        UiNumberModelUpdateFailure, UiNumberModelUpdateOrigin, UiNumberModelUpdateRequest,
        UiNumberModelUpdateStatus,
    },
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

use crate::ui::{
    dispatch::UiInputManager,
    surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface},
};

const OWNER: UiNodeId = UiNodeId::new(2);

#[test]
fn focused_bound_refresh_updates_canonical_value_without_overwriting_the_buffer() {
    let mut surface = number_field_surface(42.0, true, "-");
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.number_model_key(&mut surface, OWNER).unwrap();
    let tree_id = surface.tree.tree_id.clone();

    let applied = manager.update_number_model(
        &mut surface,
        UiNumberModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiNumberModelUpdateOrigin::BoundRefresh,
            7.0,
        ),
    );

    assert_eq!(applied.status, UiNumberModelUpdateStatus::Applied);
    assert_eq!(applied.current_model.unwrap().revision.get(), 1);
    assert_eq!(float(&surface, "value"), Some(7.0));
    assert_eq!(text(&surface, "value_text"), "-");
    assert_eq!(boolean(&surface, "number_edit_active"), Some(true));
    assert_eq!(integer(&surface, "number_value_revision"), 1);
    assert_eq!(integer(&surface, "number_edit_base_revision"), 0);
}

#[test]
fn stale_focused_enter_preserves_the_buffer_and_blur_adopts_the_model() {
    let mut surface = number_field_surface(42.0, true, "-");
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.number_model_key(&mut surface, OWNER).unwrap();
    let tree_id = surface.tree.tree_id.clone();
    assert_eq!(
        manager
            .update_number_model(
                &mut surface,
                UiNumberModelUpdateRequest::new(
                    tree_id,
                    OWNER,
                    expected,
                    UiNumberModelUpdateOrigin::BoundRefresh,
                    7.0,
                ),
            )
            .status,
        UiNumberModelUpdateStatus::Applied
    );

    let enter = dispatch_key(&mut manager, &mut surface, "Enter", 13);

    assert_eq!(
        enter
            .diagnostics
            .number_input
            .map(|receipt| receipt.commit_status),
        Some(UiNumberInputCommitStatus::Conflict)
    );
    assert_eq!(float(&surface, "value"), Some(7.0));
    assert_eq!(text(&surface, "value_text"), "-");
    assert_eq!(boolean(&surface, "number_edit_active"), Some(true));
    assert_eq!(integer(&surface, "number_edit_base_revision"), 0);

    surface.clear_focus();

    assert_eq!(float(&surface, "value"), Some(7.0));
    assert_eq!(text(&surface, "value_text"), "7");
    assert_eq!(boolean(&surface, "number_edit_active"), Some(false));
    assert_eq!(integer(&surface, "number_value_revision"), 1);
    assert_eq!(integer(&surface, "number_edit_base_revision"), 1);
}

#[test]
fn explicit_set_value_replaces_a_focused_buffer_and_closes_edit_mode() {
    let mut surface = number_field_surface(42.0, true, "-");
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.number_model_key(&mut surface, OWNER).unwrap();
    let tree_id = surface.tree.tree_id.clone();

    let applied = manager.update_number_model(
        &mut surface,
        UiNumberModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiNumberModelUpdateOrigin::ExplicitSetValue,
            9.0,
        ),
    );

    assert_eq!(applied.status, UiNumberModelUpdateStatus::Applied);
    assert_eq!(float(&surface, "value"), Some(9.0));
    assert_eq!(text(&surface, "value_text"), "9");
    assert_eq!(boolean(&surface, "number_edit_active"), Some(false));
    assert_eq!(integer(&surface, "number_value_revision"), 1);
    assert_eq!(integer(&surface, "number_edit_base_revision"), 1);
}

#[test]
fn stale_model_request_conflicts_without_partial_state() {
    let mut surface = number_field_surface(42.0, false, "42");
    let mut manager = UiInputManager::default();
    let stale = manager.number_model_key(&mut surface, OWNER).unwrap();
    let tree_id = surface.tree.tree_id.clone();
    assert_eq!(
        manager
            .update_number_model(
                &mut surface,
                UiNumberModelUpdateRequest::new(
                    tree_id.clone(),
                    OWNER,
                    stale,
                    UiNumberModelUpdateOrigin::BoundRefresh,
                    7.0,
                ),
            )
            .status,
        UiNumberModelUpdateStatus::Applied
    );
    let attributes_before = attributes(&surface).clone();

    let conflict = manager.update_number_model(
        &mut surface,
        UiNumberModelUpdateRequest::new(
            tree_id,
            OWNER,
            stale,
            UiNumberModelUpdateOrigin::BoundRefresh,
            8.0,
        ),
    );

    assert_eq!(conflict.status, UiNumberModelUpdateStatus::Conflict);
    assert_eq!(
        conflict.failure,
        Some(UiNumberModelUpdateFailure::StaleModel)
    );
    assert_eq!(attributes(&surface), &attributes_before);
}

#[test]
fn non_finite_request_is_content_free_and_zero_write() {
    let mut surface = number_field_surface(42.0, false, "42");
    let mut manager = UiInputManager::default();
    let expected = manager.number_model_key(&mut surface, OWNER).unwrap();
    let attributes_before = attributes(&surface).clone();
    let tree_id = surface.tree.tree_id.clone();

    let rejected = manager.update_number_model(
        &mut surface,
        UiNumberModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiNumberModelUpdateOrigin::BoundRefresh,
            f64::NAN,
        ),
    );

    assert_eq!(rejected.status, UiNumberModelUpdateStatus::Rejected);
    assert_eq!(
        rejected.failure,
        Some(UiNumberModelUpdateFailure::NonFiniteValue)
    );
    assert_eq!(rejected.validate(), Ok(()));
    assert!(!format!("{rejected:?}").contains("NaN"));
    assert_eq!(attributes(&surface), &attributes_before);
}

#[test]
fn exhausted_revision_rejects_a_real_change_before_writing() {
    let mut surface = number_field_surface(42.0, false, "42");
    let metadata = surface
        .tree
        .node_mut(OWNER)
        .and_then(|node| node.template_metadata.as_mut())
        .expect("number field metadata");
    metadata.attributes.insert(
        "number_value_revision".to_string(),
        toml::Value::Integer(i64::MAX),
    );
    metadata.attributes.insert(
        "number_edit_base_revision".to_string(),
        toml::Value::Integer(i64::MAX),
    );
    let mut manager = UiInputManager::default();
    let expected = manager.number_model_key(&mut surface, OWNER).unwrap();
    let attributes_before = attributes(&surface).clone();
    let tree_id = surface.tree.tree_id.clone();

    let rejected = manager.update_number_model(
        &mut surface,
        UiNumberModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiNumberModelUpdateOrigin::BoundRefresh,
            43.0,
        ),
    );

    assert_eq!(rejected.status, UiNumberModelUpdateStatus::Rejected);
    assert_eq!(
        rejected.failure,
        Some(UiNumberModelUpdateFailure::RevisionExhausted)
    );
    assert_eq!(attributes(&surface), &attributes_before);
}

#[test]
fn malformed_edit_authority_rejects_before_writing() {
    let mut surface = number_field_surface(42.0, false, "42");
    let mut manager = UiInputManager::default();
    let expected = manager.number_model_key(&mut surface, OWNER).unwrap();
    surface
        .tree
        .node_mut(OWNER)
        .and_then(|node| node.template_metadata.as_mut())
        .expect("number field metadata")
        .attributes
        .insert(
            "number_edit_active".to_string(),
            toml::Value::String("invalid".to_string()),
        );
    let attributes_before = attributes(&surface).clone();
    let tree_id = surface.tree.tree_id.clone();

    let rejected = manager.update_number_model(
        &mut surface,
        UiNumberModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiNumberModelUpdateOrigin::BoundRefresh,
            43.0,
        ),
    );

    assert_eq!(rejected.status, UiNumberModelUpdateStatus::Rejected);
    assert_eq!(
        rejected.failure,
        Some(UiNumberModelUpdateFailure::InvalidTarget)
    );
    assert_eq!(attributes(&surface), &attributes_before);
}

#[test]
fn model_identity_changes_when_the_manager_switches_surface_sessions() {
    let mut first = number_field_surface(42.0, false, "42");
    let mut second = number_field_surface(42.0, false, "42");
    let mut manager = UiInputManager::default();

    let first_key = manager.number_model_key(&mut first, OWNER).unwrap();
    let second_key = manager.number_model_key(&mut second, OWNER).unwrap();

    assert_ne!(first_key.model_id, second_key.model_id);
    assert_eq!(first_key.revision, second_key.revision);
}

#[test]
fn model_identity_changes_when_a_retained_node_id_is_reused() {
    let mut surface = number_field_surface(42.0, false, "42");
    let mut manager = UiInputManager::default();
    let before = manager.number_model_key(&mut surface, OWNER).unwrap();
    let tree_id = surface.tree.tree_id.clone();

    surface.detach_subtree_to_pool(OWNER).unwrap();
    surface
        .insert_or_reuse_pooled_child(UiNodeId::new(1), number_field_node(42.0, false, "42"))
        .unwrap();
    let after = manager.number_model_key(&mut surface, OWNER).unwrap();

    assert_ne!(after.model_id, before.model_id);
    assert_eq!(after.revision, before.revision);

    let attributes_before = attributes(&surface).clone();
    let conflict = manager.update_number_model(
        &mut surface,
        UiNumberModelUpdateRequest::new(
            tree_id,
            OWNER,
            before,
            UiNumberModelUpdateOrigin::BoundRefresh,
            7.0,
        ),
    );
    assert_eq!(conflict.status, UiNumberModelUpdateStatus::Conflict);
    assert_eq!(
        conflict.failure,
        Some(UiNumberModelUpdateFailure::StaleModel)
    );
    assert_eq!(attributes(&surface), &attributes_before);
}

#[test]
fn model_identity_survives_unrelated_sibling_topology_changes() {
    let mut surface = number_field_surface(42.0, false, "42");
    let mut manager = UiInputManager::default();
    let before = manager.number_model_key(&mut surface, OWNER).unwrap();

    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/unrelated"))
                .with_frame(UiFrame::new(8.0, 48.0, 120.0, 24.0)),
        )
        .unwrap();
    let after_insert = manager.number_model_key(&mut surface, OWNER).unwrap();
    surface.detach_subtree_to_pool(UiNodeId::new(3)).unwrap();
    let after_detach = manager.number_model_key(&mut surface, OWNER).unwrap();

    assert_eq!(after_insert, before);
    assert_eq!(after_detach, before);
}

#[test]
fn model_key_rejects_a_non_float_canonical_value() {
    let mut surface = number_field_surface(42.0, false, "42");
    surface
        .tree
        .node_mut(OWNER)
        .and_then(|node| node.template_metadata.as_mut())
        .expect("number field metadata")
        .attributes
        .insert("value".to_string(), toml::Value::Integer(42));
    let mut manager = UiInputManager::default();

    assert_eq!(
        manager.number_model_key(&mut surface, OWNER),
        Err(UiNumberModelUpdateFailure::InvalidTarget)
    );
}

#[test]
fn generic_mutation_cannot_bypass_number_field_internal_state() {
    let mut surface = number_field_surface(42.0, true, "-");
    let attributes_before = attributes(&surface).clone();
    let attempts = [
        ("value_text", UiValue::String("bypass".to_string())),
        ("number_edit_active", UiValue::Bool(false)),
        ("number_value_revision", UiValue::Int(99)),
        ("number_edit_base_revision", UiValue::Int(99)),
    ];

    for (property, value) in attempts {
        let report = surface
            .mutate_property(UiPropertyMutationRequest::new(OWNER, property, value))
            .unwrap();
        assert_eq!(report.status, UiPropertyMutationStatus::Rejected);
    }

    assert_eq!(attributes(&surface), &attributes_before);
}

#[test]
fn generic_canonical_value_mutation_advances_the_numeric_model_revision() {
    let mut surface = number_field_surface(42.0, false, "42");
    let mut manager = UiInputManager::default();
    let before = manager.number_model_key(&mut surface, OWNER).unwrap();

    let report = surface
        .mutate_property(UiPropertyMutationRequest::new(
            OWNER,
            "value",
            UiValue::Float(43.0),
        ))
        .unwrap();
    let after = manager.number_model_key(&mut surface, OWNER).unwrap();

    assert_eq!(report.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(after.model_id, before.model_id);
    assert_eq!(after.revision.get(), before.revision.get() + 1);
    assert_eq!(integer(&surface, "number_value_revision"), 1);
    assert_eq!(integer(&surface, "number_edit_base_revision"), 1);
}

fn dispatch_key(
    manager: &mut UiInputManager,
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    manager
        .dispatch_input_event(
            surface,
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(30),
                    UiInputSequence::new(3),
                ),
                state: UiKeyboardInputState::Pressed,
                key_code,
                scan_code: None,
                physical_key: logical_key.to_string(),
                logical_key: logical_key.to_string(),
                text: None,
            }),
        )
        .unwrap()
}

fn number_field_surface(value: f64, edit_active: bool, value_text: &str) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.number_model_update"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 120.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            number_field_node(value, edit_active, value_text),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn number_field_node(value: f64, edit_active: bool, value_text: &str) -> UiTreeNode {
    let mut attributes = toml::map::Map::new();
    attributes.insert("value".to_string(), toml::Value::Float(value));
    attributes.insert(
        "value_text".to_string(),
        toml::Value::String(value_text.to_string()),
    );
    attributes.insert(
        "number_edit_active".to_string(),
        toml::Value::Boolean(edit_active),
    );
    attributes.insert("number_value_revision".to_string(), toml::Value::Integer(0));
    attributes.insert(
        "number_edit_base_revision".to_string(),
        toml::Value::Integer(0),
    );
    attributes.insert(
        "caret_offset".to_string(),
        toml::Value::Integer(value_text.len() as i64),
    );
    attributes.insert("min".to_string(), toml::Value::Float(0.0));
    attributes.insert("max".to_string(), toml::Value::Float(100.0));
    attributes.insert("step".to_string(), toml::Value::Float(1.0));
    UiTreeNode::new(OWNER, UiNodePath::new("root/input"))
        .with_frame(UiFrame::new(8.0, 8.0, 240.0, 32.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(focusable_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "NumberField".to_string(),
            attributes,
            widget: UiWidgetContract {
                behavior: UiWidgetBehavior::TextInput,
                value: Some(UiValue::Float(value)),
                value_property: Some("value".to_string()),
                ..UiWidgetContract::default()
            },
            ..UiTemplateNodeMetadata::default()
        })
}

fn attributes(surface: &UiSurface) -> &toml::map::Map<String, toml::Value> {
    &surface
        .tree
        .node(OWNER)
        .and_then(|node| node.template_metadata.as_ref())
        .expect("number field metadata")
        .attributes
}

fn text<'surface>(surface: &'surface UiSurface, property: &str) -> &'surface str {
    attributes(surface)
        .get(property)
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
}

fn float(surface: &UiSurface, property: &str) -> Option<f64> {
    attributes(surface)
        .get(property)
        .and_then(toml::Value::as_float)
}

fn integer(surface: &UiSurface, property: &str) -> i64 {
    attributes(surface)
        .get(property)
        .and_then(toml::Value::as_integer)
        .unwrap_or_default()
}

fn boolean(surface: &UiSurface, property: &str) -> Option<bool> {
    attributes(surface)
        .get(property)
        .and_then(toml::Value::as_bool)
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        enabled: true,
        visible: true,
        focusable: true,
        ..UiStateFlags::default()
    }
}

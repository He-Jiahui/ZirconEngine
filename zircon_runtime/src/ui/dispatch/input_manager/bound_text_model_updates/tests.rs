use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiTextInputEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    text::{
        UiTextDocumentId, UiTextModelUpdateFailure, UiTextModelUpdateId, UiTextModelUpdateOrigin,
        UiTextModelUpdateRequest, UiTextModelUpdateStatus,
    },
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

use crate::ui::{
    dispatch::UiInputManager,
    surface::{UiPropertyMutationRequest, UiSurface},
};

const OWNER: UiNodeId = UiNodeId::new(2);

#[test]
fn malformed_request_returns_a_valid_content_free_rejection() {
    let mut surface = text_input_surface("editable", false);
    let mut manager = UiInputManager::default();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();
    let mut request = UiTextModelUpdateRequest::new(
        surface.tree.tree_id.clone(),
        OWNER,
        expected,
        UiTextModelUpdateOrigin::BoundRefresh,
        "malformed-sensitive-value",
    );
    request.schema_version = 0;
    request.request_id = UiTextModelUpdateId::default();
    request.expected_document.document_id = UiTextDocumentId::default();

    let rejected = manager.update_text_model(&mut surface, request);

    assert_eq!(rejected.status, UiTextModelUpdateStatus::Rejected);
    assert_eq!(
        rejected.failure,
        Some(UiTextModelUpdateFailure::UnsupportedSchemaVersion)
    );
    assert_eq!(rejected.validate(), Ok(()));
    assert!(!format!("{rejected:?}").contains("malformed-sensitive-value"));
    assert_eq!(text(&surface), "editable");
}

#[test]
fn focused_bound_refresh_defers_without_overwriting_the_edit_buffer() {
    let mut surface = text_input_surface("editable", false);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();

    let tree_id = surface.tree.tree_id.clone();
    let deferred = manager.update_text_model(
        &mut surface,
        UiTextModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiTextModelUpdateOrigin::BoundRefresh,
            "model-refresh",
        ),
    );

    assert_eq!(deferred.status, UiTextModelUpdateStatus::Deferred);
    assert_eq!(text(&surface), "editable");
    assert!(manager.drain_text_model_update_receipts().is_empty());
}

#[test]
fn latest_unchanged_refresh_supersedes_an_older_deferred_value() {
    let mut surface = text_input_surface("editable", false);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();
    let stale = UiTextModelUpdateRequest::new(
        surface.tree.tree_id.clone(),
        OWNER,
        expected,
        UiTextModelUpdateOrigin::BoundRefresh,
        "stale-model-value",
    );
    let stale_id = stale.request_id;
    assert_eq!(
        manager.update_text_model(&mut surface, stale).status,
        UiTextModelUpdateStatus::Deferred
    );

    let latest = manager.update_text_model(
        &mut surface,
        UiTextModelUpdateRequest::new(
            surface.tree.tree_id.clone(),
            OWNER,
            expected,
            UiTextModelUpdateOrigin::BoundRefresh,
            "editable",
        ),
    );
    assert_eq!(latest.status, UiTextModelUpdateStatus::Unchanged);
    let receipts = manager.drain_text_model_update_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].request_id, stale_id);
    assert_eq!(
        receipts[0].failure,
        Some(UiTextModelUpdateFailure::Superseded)
    );

    surface.clear_focus();
    manager
        .tick(&mut surface, UiInputTimestamp::from_micros(10))
        .unwrap();
    assert_eq!(text(&surface), "editable");
    assert!(manager.drain_text_model_update_receipts().is_empty());
}

#[test]
fn unfocused_bound_refresh_rebases_the_document_and_surface_once() {
    let mut surface = text_input_surface("editable", false);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();
    let request = UiTextModelUpdateRequest::new(
        surface.tree.tree_id.clone(),
        OWNER,
        expected,
        UiTextModelUpdateOrigin::BoundRefresh,
        "model-refresh",
    );
    let request_id = request.request_id;
    assert_eq!(
        manager.update_text_model(&mut surface, request).status,
        UiTextModelUpdateStatus::Deferred
    );

    surface.clear_focus();
    manager
        .tick(&mut surface, UiInputTimestamp::from_micros(10))
        .unwrap();

    assert_eq!(text(&surface), "model-refresh");
    let receipts = manager.drain_text_model_update_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].request_id, request_id);
    assert_eq!(receipts[0].status, UiTextModelUpdateStatus::Applied);
    let edit = receipts[0].document_edit.as_ref().expect("document edit");
    assert_eq!(edit.previous_revision, expected.revision);
    assert_eq!(edit.revision.get(), expected.revision.get() + 1);
}

#[test]
fn user_edit_after_defer_wins_and_blur_emits_content_free_conflict() {
    let mut surface = text_input_surface("editable", false);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();
    let request = UiTextModelUpdateRequest::new(
        surface.tree.tree_id.clone(),
        OWNER,
        expected,
        UiTextModelUpdateOrigin::BoundRefresh,
        "stale-model-value",
    );
    let request_id = request.request_id;
    assert_eq!(
        manager.update_text_model(&mut surface, request).status,
        UiTextModelUpdateStatus::Deferred
    );

    manager
        .dispatch_input_event(
            &mut surface,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(20),
                    UiInputSequence::new(1),
                ),
                text: "!".to_string(),
            }),
        )
        .unwrap();
    assert_eq!(text(&surface), "editable!");
    surface.clear_focus();
    manager
        .tick(&mut surface, UiInputTimestamp::from_micros(30))
        .unwrap();

    assert_eq!(text(&surface), "editable!");
    let receipts = manager.drain_text_model_update_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].request_id, request_id);
    assert_eq!(receipts[0].status, UiTextModelUpdateStatus::Conflict);
    assert_eq!(
        receipts[0].failure,
        Some(UiTextModelUpdateFailure::StaleDocument)
    );
    assert!(receipts[0].document_edit.is_none());
    assert!(format!("{:?}", receipts[0]).contains("StaleDocument"));
    assert!(!format!("{:?}", receipts[0]).contains("stale-model-value"));
}

#[test]
fn explicit_set_text_forces_focused_replace_and_moves_caret_to_end() {
    let mut surface = text_input_surface("editable", false);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();

    let tree_id = surface.tree.tree_id.clone();
    let applied = manager.update_text_model(
        &mut surface,
        UiTextModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiTextModelUpdateOrigin::ExplicitSetText,
            "forced",
        ),
    );

    assert_eq!(applied.status, UiTextModelUpdateStatus::Applied);
    assert_eq!(text(&surface), "forced");
    assert_eq!(integer(&surface, "caret_offset"), 6);
    assert!(applied.document_edit.is_some());
}

#[test]
fn explicit_load_text_uses_the_same_focused_force_review_boundary() {
    let mut surface = text_input_surface("editable", false);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();

    let applied = manager.update_text_model(
        &mut surface,
        UiTextModelUpdateRequest::new(
            surface.tree.tree_id.clone(),
            OWNER,
            expected,
            UiTextModelUpdateOrigin::ExplicitLoadText,
            "loaded",
        ),
    );

    assert_eq!(applied.status, UiTextModelUpdateStatus::Applied);
    assert_eq!(text(&surface), "loaded");
    assert_eq!(integer(&surface, "caret_offset"), 6);
    assert!(applied.document_edit.is_some());
}

#[test]
fn explicit_set_text_during_preedit_replaces_the_committed_document_once() {
    let mut surface = text_input_surface("editable", false);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    surface.input.input_method_owner = Some(OWNER);
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();

    manager
        .dispatch_input_event(
            &mut surface,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(35),
                    UiInputSequence::new(2),
                ),
                kind: UiImeInputEventKind::Preedit,
                text: "XY".to_string(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap();
    assert_eq!(text(&surface), "editableXY");

    let applied = manager.update_text_model(
        &mut surface,
        UiTextModelUpdateRequest::new(
            surface.tree.tree_id.clone(),
            OWNER,
            expected,
            UiTextModelUpdateOrigin::ExplicitSetText,
            "forced",
        ),
    );

    assert_eq!(applied.status, UiTextModelUpdateStatus::Applied);
    assert_eq!(text(&surface), "forced");
    assert_eq!(integer(&surface, "caret_offset"), 6);
    assert_eq!(integer(&surface, "composition_start"), 6);
    assert_eq!(integer(&surface, "composition_end"), 6);
    assert_eq!(text_property(&surface, "composition_text"), "");
    assert_eq!(text_property(&surface, "composition_restore_text"), "");
    let edit = applied.document_edit.as_ref().expect("document edit");
    assert_eq!(edit.previous_revision, expected.revision);
    assert_eq!(edit.revision.get(), expected.revision.get() + 1);
}

#[test]
fn secure_deferred_value_lives_in_the_surface_secure_store() {
    let mut surface = text_input_surface("secret", true);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();

    let tree_id = surface.tree.tree_id.clone();
    let deferred = manager.update_text_model(
        &mut surface,
        UiTextModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiTextModelUpdateOrigin::BoundRefresh,
            "pending-secure-model",
        ),
    );

    assert_eq!(deferred.status, UiTextModelUpdateStatus::Deferred);
    let pending = surface
        .take_pending_secure_text_model_update(OWNER)
        .expect("secure pending value");
    assert_eq!(pending, "pending-secure-model");
    surface.store_pending_secure_text_model_update(OWNER, pending);
    surface.clear_focus();
    manager
        .tick(&mut surface, UiInputTimestamp::from_micros(40))
        .unwrap();
    assert_eq!(text(&surface), "pending-secure-model");
}

#[test]
fn secure_policy_change_rejects_and_discards_the_pending_value() {
    let mut surface = text_input_surface("secret", true);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();
    let request = UiTextModelUpdateRequest::new(
        surface.tree.tree_id.clone(),
        OWNER,
        expected,
        UiTextModelUpdateOrigin::BoundRefresh,
        "pending-policy-secret",
    );
    let request_id = request.request_id;
    assert_eq!(
        manager.update_text_model(&mut surface, request).status,
        UiTextModelUpdateStatus::Deferred
    );

    surface
        .mutate_property(UiPropertyMutationRequest::new(
            OWNER,
            "secure",
            UiValue::Bool(false),
        ))
        .unwrap();
    manager
        .tick(&mut surface, UiInputTimestamp::from_micros(45))
        .unwrap();

    assert!(surface
        .take_pending_secure_text_model_update(OWNER)
        .is_none());
    let receipts = manager.drain_text_model_update_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].request_id, request_id);
    assert_eq!(
        receipts[0].failure,
        Some(UiTextModelUpdateFailure::SecurityPolicyChanged)
    );
    assert!(!format!("{:?}", receipts[0]).contains("pending-policy-secret"));
}

#[test]
fn detached_secure_owner_rejects_and_discards_the_pending_value() {
    let mut surface = text_input_surface("secret", true);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();
    let request = UiTextModelUpdateRequest::new(
        surface.tree.tree_id.clone(),
        OWNER,
        expected,
        UiTextModelUpdateOrigin::BoundRefresh,
        "pending-detached-secret",
    );
    let request_id = request.request_id;
    assert_eq!(
        manager.update_text_model(&mut surface, request).status,
        UiTextModelUpdateStatus::Deferred
    );

    surface.tree.nodes.remove(&OWNER);
    manager
        .tick(&mut surface, UiInputTimestamp::from_micros(50))
        .unwrap();

    assert!(surface
        .take_pending_secure_text_model_update(OWNER)
        .is_none());
    let receipts = manager.drain_text_model_update_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].request_id, request_id);
    assert_eq!(
        receipts[0].failure,
        Some(UiTextModelUpdateFailure::OwnerDetached)
    );
    assert!(!format!("{:?}", receipts[0]).contains("pending-detached-secret"));
}

#[test]
fn surface_switch_revokes_the_previous_surface_pending_secure_value() {
    let mut first = text_input_surface("first-secret", true);
    let mut second = text_input_surface("second-secret", true);
    let mut manager = UiInputManager::default();
    first.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut first, OWNER).unwrap();
    let request = UiTextModelUpdateRequest::new(
        first.tree.tree_id.clone(),
        OWNER,
        expected,
        UiTextModelUpdateOrigin::BoundRefresh,
        "pending-surface-secret",
    );
    let request_id = request.request_id;
    assert_eq!(
        manager.update_text_model(&mut first, request).status,
        UiTextModelUpdateStatus::Deferred
    );

    manager.text_document_key(&mut second, OWNER).unwrap();

    assert!(first.take_pending_secure_text_model_update(OWNER).is_none());
    let receipts = manager.drain_text_model_update_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].request_id, request_id);
    assert_eq!(
        receipts[0].failure,
        Some(UiTextModelUpdateFailure::OwnerDetached)
    );
}

#[test]
fn manager_drop_revokes_pending_secure_values_from_the_surface_store() {
    let mut surface = text_input_surface("secret", true);
    surface.focus_node(OWNER).unwrap();
    {
        let mut manager = UiInputManager::default();
        let expected = manager.text_document_key(&mut surface, OWNER).unwrap();
        let request = UiTextModelUpdateRequest::new(
            surface.tree.tree_id.clone(),
            OWNER,
            expected,
            UiTextModelUpdateOrigin::BoundRefresh,
            "pending-manager-drop-secret",
        );
        assert_eq!(
            manager.update_text_model(&mut surface, request).status,
            UiTextModelUpdateStatus::Deferred
        );
    }

    assert!(surface
        .take_pending_secure_text_model_update(OWNER)
        .is_none());
}

#[test]
fn oversized_focused_refresh_is_rejected_without_retention_or_mutation() {
    let mut surface = text_input_surface("editable", true);
    let mut manager = UiInputManager::default();
    surface.focus_node(OWNER).unwrap();
    let expected = manager.text_document_key(&mut surface, OWNER).unwrap();

    let tree_id = surface.tree.tree_id.clone();
    let rejected = manager.update_text_model(
        &mut surface,
        UiTextModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiTextModelUpdateOrigin::BoundRefresh,
            "x".repeat(super::MVP_MAX_TEXT_MODEL_UPDATE_VALUE_BYTES + 1),
        ),
    );

    assert_eq!(rejected.status, UiTextModelUpdateStatus::Rejected);
    assert_eq!(
        rejected.failure,
        Some(UiTextModelUpdateFailure::ValueTooLarge)
    );
    assert_eq!(text(&surface), "editable");
    assert!(surface
        .take_pending_secure_text_model_update(OWNER)
        .is_none());
}

#[test]
fn string_model_gateway_rejects_number_field_without_touching_typed_or_edit_value() {
    let mut manager = UiInputManager::default();
    let mut text_surface = text_input_surface("source", false);
    let expected = manager.text_document_key(&mut text_surface, OWNER).unwrap();
    let mut number_surface = number_field_surface(42.0);

    assert_eq!(
        manager.text_document_key(&mut number_surface, OWNER),
        Err(UiTextModelUpdateFailure::InvalidTarget)
    );
    let tree_id = number_surface.tree.tree_id.clone();
    let rejected = manager.update_text_model(
        &mut number_surface,
        UiTextModelUpdateRequest::new(
            tree_id,
            OWNER,
            expected,
            UiTextModelUpdateOrigin::BoundRefresh,
            "99",
        ),
    );

    assert_eq!(rejected.status, UiTextModelUpdateStatus::Rejected);
    assert_eq!(
        rejected.failure,
        Some(UiTextModelUpdateFailure::InvalidTarget)
    );
    assert_eq!(number_value(&number_surface), Some(42.0));
    assert_eq!(text_property(&number_surface, "value_text"), "");
}

fn text_input_surface(value: &str, secure: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.bound_text_model_update"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 120.0)),
    );
    let mut attributes = toml::map::Map::new();
    attributes.insert(
        "content".to_string(),
        toml::Value::String(value.to_string()),
    );
    attributes.insert(
        "caret_offset".to_string(),
        toml::Value::Integer(value.len() as i64),
    );
    if secure {
        attributes.insert("secure".to_string(), toml::Value::Boolean(true));
    }
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(OWNER, UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(8.0, 8.0, 240.0, 32.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "SearchBox".to_string(),
                    attributes,
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn number_field_surface(value: f64) -> UiSurface {
    let mut surface = text_input_surface(&UiValue::Float(value).display_text(), false);
    let metadata = surface
        .tree
        .node_mut(OWNER)
        .and_then(|node| node.template_metadata.as_mut())
        .expect("number field metadata");
    metadata.component = "NumberField".to_string();
    metadata.attributes.remove("content");
    metadata
        .attributes
        .insert("value".to_string(), toml::Value::Float(value));
    metadata
        .attributes
        .insert("value_text".to_string(), toml::Value::String(String::new()));
    metadata.attributes.insert(
        "number_edit_active".to_string(),
        toml::Value::Boolean(false),
    );
    metadata.widget.value = Some(UiValue::Float(value));
    metadata.widget.value_property = Some("value".to_string());
    surface.rebuild();
    surface
}

fn text(surface: &UiSurface) -> &str {
    surface
        .tree
        .node(OWNER)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("content"))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
}

fn number_value(surface: &UiSurface) -> Option<f64> {
    surface
        .tree
        .node(OWNER)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("value"))
        .and_then(toml::Value::as_float)
}

fn integer(surface: &UiSurface, property: &str) -> i64 {
    surface
        .tree
        .node(OWNER)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(property))
        .and_then(toml::Value::as_integer)
        .unwrap_or_default()
}

fn text_property<'surface>(surface: &'surface UiSurface, property: &str) -> &'surface str {
    surface
        .tree
        .node(OWNER)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(property))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        enabled: true,
        visible: true,
        focusable: true,
        ..UiStateFlags::default()
    }
}

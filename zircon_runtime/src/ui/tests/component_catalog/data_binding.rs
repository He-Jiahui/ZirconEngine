use crate::ui::component::inspector_selected_entity_data_source;
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentBindingTarget,
    UiComponentDataSourceFieldDescriptor, UiComponentDataSourceFieldOption,
    UiComponentDataSourceKind, UiComponentEvent, UiComponentEventEnvelope, UiComponentEventKind,
    UiComponentProjectionPatch, UiDragSourceMetadata, UiValidationLevel, UiValue, UiValueKind,
};

#[test]
fn component_event_envelope_preserves_typed_event_and_target() {
    let target =
        UiComponentBindingTarget::inspector("entity://selected", "transform.translation.x");
    let event = UiComponentEvent::ValueChanged {
        property: "value".to_string(),
        value: UiValue::Float(42.5),
    };
    let source = UiDragSourceMetadata::asset(
        "inspector",
        "translation-x",
        "asset-123",
        "assets/player.scene",
        "Player Scene",
        "scene",
        "scene",
    );

    let envelope = UiComponentEventEnvelope::new(
        "document-main",
        "translation-x",
        target.clone(),
        event.clone(),
    )
    .with_component_id("NumberField")
    .with_source(source.clone());

    assert_eq!(envelope.document_id, "document-main");
    assert_eq!(envelope.control_id, "translation-x");
    assert_eq!(envelope.component_id.as_deref(), Some("NumberField"));
    assert_eq!(envelope.target, target);
    assert_eq!(envelope.event_kind, UiComponentEventKind::ValueChanged);
    assert_eq!(envelope.event, event);
    assert_eq!(envelope.source, Some(source));

    let round_trip: UiComponentEventEnvelope =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
    assert_eq!(round_trip, envelope);
}

#[test]
fn component_event_envelope_rejects_mismatched_wire_event_kind() {
    let envelope = UiComponentEventEnvelope::new(
        "document-main",
        "name-field",
        UiComponentBindingTarget::inspector("entity://selected", "name"),
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("Player".to_string()),
        },
    );
    let mut wire_value = serde_json::to_value(&envelope).unwrap();
    wire_value["event_kind"] = serde_json::json!("Commit");

    let error = serde_json::from_value::<UiComponentEventEnvelope>(wire_value).unwrap_err();
    assert!(error.to_string().contains("does not match typed event"));
}

#[test]
fn component_projection_patch_keeps_attribute_and_state_values_separate() {
    let patch = UiComponentProjectionPatch::new("name-field")
        .with_attribute("value", UiValue::String("Player".to_string()))
        .with_state_value("focused", UiValue::Bool(true));

    assert_eq!(patch.control_id, "name-field");
    assert_eq!(
        patch.attributes.get("value"),
        Some(&UiValue::String("Player".to_string()))
    );
    assert_eq!(
        patch.state_values.get("focused"),
        Some(&UiValue::Bool(true))
    );
    assert!(!patch.attributes.contains_key("focused"));
    assert!(!patch.state_values.contains_key("value"));

    let result = UiComponentAdapterResult::changed()
        .with_transaction("inspector:name")
        .with_mutation_source("inspector")
        .with_status("Name updated")
        .with_patch(patch.clone());
    assert!(result.changed);
    assert!(result.dirty);
    assert!(result.refresh_projection);
    assert_eq!(result.transaction_id.as_deref(), Some("inspector:name"));
    assert_eq!(result.mutation_source.as_deref(), Some("inspector"));
    assert_eq!(result.status_text.as_deref(), Some("Name updated"));
    assert_eq!(result.patches, vec![patch]);
}

#[test]
fn component_adapter_error_reports_unsupported_target_without_editor_types() {
    let target = UiComponentBindingTarget::new("dock", "left-pane");
    let error = UiComponentAdapterError::UnsupportedTargetDomain {
        domain: target.domain,
    };

    assert_eq!(
        error.to_string(),
        "unsupported Runtime UI component target domain dock"
    );
}

#[test]
fn component_data_source_descriptor_names_required_sources_without_editor_types() {
    let descriptor = inspector_selected_entity_data_source();
    assert_eq!(descriptor.domain, "inspector");
    assert_eq!(descriptor.source_name, "subject");
    assert_eq!(descriptor.display_name, "Selected Entity Inspector");
    assert_eq!(descriptor.kind, UiComponentDataSourceKind::Inspector);
    assert_eq!(descriptor.subject.as_deref(), Some("entity://selected"));
    assert!(descriptor.writable);
    assert_eq!(
        descriptor.value_kinds,
        vec![UiValueKind::String, UiValueKind::Int, UiValueKind::Float]
    );
    assert_eq!(descriptor.fields.len(), 5);
    assert_eq!(descriptor.fields[0].path, "name");
    assert_eq!(descriptor.fields[2].group.as_deref(), Some("Transform"));
    assert_eq!(descriptor.fields[2].step, Some(0.1));

    let reflection_target =
        UiComponentBindingTarget::reflection("component://selected", "transform.translation.x");
    assert_eq!(reflection_target.domain, "reflection");
    assert_eq!(
        reflection_target.subject.as_deref(),
        Some("component://selected")
    );

    let asset_editor_target =
        UiComponentBindingTarget::asset_editor("asset://ui/main_menu", "nodes.play_button.text");
    assert_eq!(asset_editor_target.domain, "asset_editor");
    assert_eq!(
        asset_editor_target.subject.as_deref(),
        Some("asset://ui/main_menu")
    );
}

#[test]
fn component_data_source_field_descriptor_covers_property_editor_metadata() {
    let field = UiComponentDataSourceFieldDescriptor::new(
        "transform.translation.x",
        "Translation X",
        UiValueKind::Float,
    )
    .writable(true)
    .group("Transform")
    .collapsed(false)
    .range(-1000.0, 1000.0)
    .step(0.1)
    .options([
        UiComponentDataSourceFieldOption::new("local", "Local"),
        UiComponentDataSourceFieldOption::new("world", "World").disabled(true),
    ])
    .array_element_kind(UiValueKind::Float)
    .map_kinds(UiValueKind::String, UiValueKind::Float)
    .reference_kind("scene-node")
    .validation(UiValidationLevel::Warning, "outside authored prefab range");

    assert_eq!(field.path, "transform.translation.x");
    assert_eq!(field.display_name, "Translation X");
    assert_eq!(field.value_kind, UiValueKind::Float);
    assert!(field.writable);
    assert_eq!(field.group.as_deref(), Some("Transform"));
    assert_eq!(field.min, Some(-1000.0));
    assert_eq!(field.max, Some(1000.0));
    assert_eq!(field.step, Some(0.1));
    assert_eq!(field.options.len(), 2);
    assert!(field.options[1].disabled);
    assert_eq!(field.element_kind, Some(UiValueKind::Float));
    assert_eq!(field.key_kind, Some(UiValueKind::String));
    assert_eq!(field.map_value_kind, Some(UiValueKind::Float));
    assert_eq!(field.reference_kind.as_deref(), Some("scene-node"));
    assert_eq!(field.validation_level, Some(UiValidationLevel::Warning));
    assert_eq!(
        field.validation_message.as_deref(),
        Some("outside authored prefab range")
    );
}

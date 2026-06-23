use super::*;

#[test]
fn runtime_component_catalog_marks_v2_model_tiers_and_layout_roles() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    for component_id in [
        "Container",
        "Overlay",
        "ListView",
        "FlexBox",
        "HorizontalGroup",
        "VerticalGroup",
        "GridGroup",
        "CanvasBox",
        "SizeBox",
        "VirtualList",
    ] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_eq!(
            descriptor.descriptor_kind,
            UiComponentDescriptorKind::Layout,
            "{component_id} should be a v2 layout component"
        );
    }

    assert_eq!(
        registry.descriptor("Overlay").unwrap().layout_role,
        UiComponentLayoutRole::Overlay
    );
    assert_eq!(
        registry.descriptor("GridGroup").unwrap().layout_role,
        UiComponentLayoutRole::Grid
    );
    assert_eq!(
        registry.descriptor("CanvasBox").unwrap().layout_role,
        UiComponentLayoutRole::Canvas
    );
    assert_eq!(
        registry.descriptor("VirtualList").unwrap().layout_role,
        UiComponentLayoutRole::VirtualList
    );

    for component_id in [
        "AssetField",
        "ObjectField",
        "Foldout",
        "TreeView",
        "EditableTable",
        "MessageBox",
    ] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_eq!(
            descriptor.descriptor_kind,
            UiComponentDescriptorKind::EditorOnly,
            "{component_id} should be editor-only"
        );
    }

    assert_eq!(
        registry
            .descriptor("InspectorSection")
            .unwrap()
            .descriptor_kind,
        UiComponentDescriptorKind::Composite
    );
}

#[test]
fn runtime_component_descriptors_validate_palette_and_schema_contracts() {
    let valid = UiComponentDescriptor::new(
        "TestWidget",
        "Test Widget",
        UiComponentCategory::Visual,
        "test-widget",
    )
    .with_prop(UiPropSchema::new("text", UiValueKind::String))
    .default_prop("text", UiValue::String("hello".to_string()))
    .requires_host_capability(UiHostCapability::Editor)
    .requires_render_capability(UiRenderCapability::Text)
    .default_node_template(UiDefaultNodeTemplate::native("TestWidget"))
    .palette(UiPaletteMetadata::new(
        "Test Widget",
        UiComponentCategory::Visual,
        "test-widget",
        UiDefaultNodeTemplate::native("TestWidget"),
    ))
    .fallback_policy(UiWidgetFallbackPolicy::new(
        UiWidgetEditorFallback::Placeholder,
        UiWidgetRuntimeFallback::RejectNode,
    ));
    assert!(validate_component_descriptor(&valid).is_ok());

    let duplicate = valid
        .clone()
        .with_prop(UiPropSchema::new("text", UiValueKind::String));
    assert!(matches!(
        validate_component_descriptor(&duplicate),
        Err(UiComponentDescriptorError::DuplicateSchemaName { name, .. }) if name == "text"
    ));

    let missing_schema = UiComponentDescriptor::new(
        "BrokenWidget",
        "Broken Widget",
        UiComponentCategory::Visual,
        "broken-widget",
    )
    .default_prop("missing", UiValue::String("value".to_string()));
    assert!(matches!(
        validate_component_descriptor(&missing_schema),
        Err(UiComponentDescriptorError::MissingDefaultPropSchema { name, .. }) if name == "missing"
    ));

    let non_finite = UiComponentDescriptor::new(
        "NonFiniteWidget",
        "Non Finite Widget",
        UiComponentCategory::Numeric,
        "non-finite-widget",
    )
    .with_prop(
        UiPropSchema::new("value", UiValueKind::Float).default_value(UiValue::Float(f64::NAN)),
    );
    assert!(matches!(
        validate_component_descriptor(&non_finite),
        Err(UiComponentDescriptorError::NonFiniteNumber { name, .. }) if name == "value"
    ));
}

#[test]
fn runtime_component_catalog_schemas_are_normalized_and_type_consistent() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    for descriptor in registry.descriptors() {
        assert_unique_schema_names(descriptor, "prop", &descriptor.prop_schema);
        assert_unique_schema_names(descriptor, "state", &descriptor.state_schema);
        assert_unique_slot_names(descriptor);
        assert_unique_events(descriptor);

        for schema in &descriptor.prop_schema {
            assert!(
                descriptor.prop(&schema.name).is_some(),
                "component {} prop lookup should find schema `{}`",
                descriptor.id,
                schema.name
            );
        }

        for schema in &descriptor.state_schema {
            assert!(
                descriptor.state_prop(&schema.name).is_some(),
                "component {} state lookup should find schema `{}`",
                descriptor.id,
                schema.name
            );
        }

        for slot in &descriptor.slot_schema {
            assert!(
                descriptor.slot_schema(&slot.name).is_some(),
                "component {} slot lookup should find schema `{}`",
                descriptor.id,
                slot.name
            );
        }

        for (name, value) in &descriptor.default_props {
            let schema = descriptor.prop(name).unwrap_or_else(|| {
                panic!(
                    "component {} default prop `{}` must have a matching prop schema",
                    descriptor.id, name
                )
            });
            assert_value_matches_schema_kind(descriptor, name, schema.value_kind, value);
        }

        for schema in descriptor
            .prop_schema
            .iter()
            .chain(descriptor.state_schema.iter())
        {
            if let Some(default_value) = &schema.default_value {
                assert_value_matches_schema_kind(
                    descriptor,
                    &schema.name,
                    schema.value_kind,
                    default_value,
                );
            }

            if let (Some(min), Some(max)) = (schema.min, schema.max) {
                assert!(
                    min <= max,
                    "component {} schema `{}` has inverted range {min}..{max}",
                    descriptor.id,
                    schema.name
                );
            }

            if let Some(step) = schema.step {
                assert!(
                    step > 0.0,
                    "component {} schema `{}` must use a positive step, got {step}",
                    descriptor.id,
                    schema.name
                );
            }
        }
    }
}

mod catalog_inventory;
mod complex_components;
mod component_state;
mod data_binding;
mod descriptor_contracts;
mod material_foundation;
mod registry_queries;
mod selection_state;

use std::collections::BTreeSet;

use crate::ui::component::{
    validate_component_descriptor, UiComponentDescriptorError, UiComponentDescriptorRegistry,
};
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentDescriptorKind, UiComponentEvent,
    UiComponentEventKind, UiComponentLayoutRole, UiComponentState, UiDefaultNodeTemplate,
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata, UiHostCapability, UiHostCapabilitySet,
    UiPaletteMetadata, UiPropSchema, UiRenderCapability, UiValidationLevel, UiValue, UiValueKind,
    UiWidgetEditorFallback, UiWidgetFallbackPolicy, UiWidgetRuntimeFallback,
};

fn test_asset_source() -> UiDragSourceMetadata {
    UiDragSourceMetadata::asset(
        "browser",
        "AssetBrowserContentPanel",
        "asset-uuid-1",
        "res://textures/grid.albedo.png",
        "Grid Albedo",
        "Texture",
        "png",
    )
}

fn assert_has_state(descriptor: &UiComponentDescriptor, name: &str) {
    assert!(
        descriptor.state_prop(name).is_some(),
        "component {} missing state schema entry `{}`",
        descriptor.id,
        name
    );
}

fn assert_has_prop(descriptor: &UiComponentDescriptor, name: &str) {
    assert!(
        descriptor.prop(name).is_some(),
        "component {} missing prop schema entry `{}`",
        descriptor.id,
        name
    );
}

fn assert_has_event(descriptor: &UiComponentDescriptor, event: UiComponentEventKind) {
    assert!(
        descriptor.supports_event(event),
        "component {} missing event support {:?}",
        descriptor.id,
        event
    );
}

fn assert_category_component_ids(
    registry: &UiComponentDescriptorRegistry,
    category: UiComponentCategory,
    expected_ids: &[&str],
) {
    assert_eq!(
        registry
            .descriptors_in_category(category)
            .map(|descriptor| descriptor.id.as_str())
            .collect::<BTreeSet<_>>(),
        expected_ids.iter().copied().collect::<BTreeSet<_>>(),
        "component category {category:?} should expose the expected V1 component ids"
    );
}

fn assert_unique_schema_names(
    descriptor: &UiComponentDescriptor,
    schema_label: &str,
    schemas: &[UiPropSchema],
) {
    let mut names = BTreeSet::new();
    for schema in schemas {
        assert!(
            names.insert(schema.name.as_str()),
            "component {} has duplicate {} schema `{}`",
            descriptor.id,
            schema_label,
            schema.name
        );
    }
}

fn assert_unique_slot_names(descriptor: &UiComponentDescriptor) {
    let mut names = BTreeSet::new();
    for slot in &descriptor.slot_schema {
        assert!(
            names.insert(slot.name.as_str()),
            "component {} has duplicate slot schema `{}`",
            descriptor.id,
            slot.name
        );
    }
}

fn assert_unique_events(descriptor: &UiComponentDescriptor) {
    let mut events = BTreeSet::new();
    for event in &descriptor.events {
        assert!(
            events.insert(format!("{event:?}")),
            "component {} has duplicate event {:?}",
            descriptor.id,
            event
        );
    }
}

fn assert_value_matches_schema_kind(
    descriptor: &UiComponentDescriptor,
    name: &str,
    expected_kind: UiValueKind,
    value: &UiValue,
) {
    if expected_kind == UiValueKind::Any {
        return;
    }

    assert_eq!(
        value.kind(),
        expected_kind,
        "component {} schema `{}` default value kind mismatch",
        descriptor.id,
        name
    );
}

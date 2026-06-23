use super::*;

#[test]
fn runtime_component_registry_filters_by_host_capabilities_and_reports_missing() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let runtime_basic = UiHostCapabilitySet::runtime_basic();

    assert!(registry
        .descriptors_for_host(&runtime_basic)
        .iter()
        .all(|descriptor| runtime_basic.contains_all(&descriptor.required_host_capabilities)));
    assert!(registry
        .descriptors_for_host(&runtime_basic)
        .iter()
        .all(|descriptor| descriptor.id != "TextField"));
    assert!(registry
        .descriptors_for_host(&runtime_basic)
        .iter()
        .all(|descriptor| descriptor.id != "WorldSpaceSurface"));

    assert!(registry
        .descriptors_for_host(&runtime_basic)
        .iter()
        .any(|descriptor| descriptor.id == "Button"));

    let missing = registry
        .missing_capabilities("TextField", &runtime_basic)
        .expect("TextField descriptor should exist");
    assert!(missing.contains(&UiHostCapability::TextInput));

    let missing_world = registry
        .missing_capabilities("WorldSpaceSurface", &runtime_basic)
        .expect("WorldSpaceSurface descriptor should exist");
    assert!(missing_world.contains(&UiHostCapability::WorldSpaceUi));

    let runtime_world_space = UiHostCapabilitySet::runtime_world_space();
    assert!(registry
        .descriptors_for_host(&runtime_world_space)
        .iter()
        .any(|descriptor| descriptor.id == "WorldSpaceSurface"));
}

#[test]
fn runtime_component_registry_builds_descriptor_palette_views() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let entries = registry.palette_entries_for_host(&UiHostCapabilitySet::editor_authoring());

    assert!(entries.windows(2).all(|window| {
        (
            window[0].category,
            &window[0].sort_key,
            &window[0].component_id,
        ) <= (
            window[1].category,
            &window[1].sort_key,
            &window[1].component_id,
        )
    }));
    let button = entries
        .iter()
        .find(|entry| entry.component_id == "Button")
        .expect("Button should be palette-visible");
    assert_eq!(button.display_name, "Button");
    assert_eq!(button.category, UiComponentCategory::Input);
    assert_eq!(button.default_node.widget_type, "Button");
    assert_eq!(
        button
            .default_node
            .props
            .get("text")
            .and_then(toml::Value::as_str),
        Some("Button")
    );

    let virtual_list = registry.descriptor("VirtualList").unwrap();
    assert!(virtual_list
        .required_render_capabilities
        .contains(&UiRenderCapability::VirtualizedLayout));
    assert_eq!(
        virtual_list.fallback_policy.runtime,
        UiWidgetRuntimeFallback::RejectNode
    );

    let container = entries
        .iter()
        .find(|entry| entry.component_id == "Container")
        .expect("Container should be palette-visible from descriptor metadata");
    assert_eq!(container.category, UiComponentCategory::Container);
    assert_eq!(container.default_node.widget_type, "Container");
    assert_eq!(
        container
            .default_node
            .layout
            .as_ref()
            .and_then(|layout| layout.get("container"))
            .and_then(toml::Value::as_table)
            .and_then(|container| container.get("kind"))
            .and_then(toml::Value::as_str),
        Some("Container")
    );
    assert!(registry
        .descriptor("Container")
        .and_then(|descriptor| descriptor.slot_schema("content"))
        .is_some());
    assert!(registry
        .descriptor("Space")
        .and_then(|descriptor| descriptor.slot_schema("content"))
        .is_none());
}

#[test]
fn runtime_component_registry_revision_changes_only_for_descriptor_set_changes() {
    let mut registry = UiComponentDescriptorRegistry::new();
    let descriptor = UiComponentDescriptor::new(
        "RevisionWidget",
        "Revision Widget",
        UiComponentCategory::Visual,
        "revision-widget",
    )
    .default_node_template(UiDefaultNodeTemplate::native("RevisionWidget"))
    .palette(UiPaletteMetadata::new(
        "Revision Widget",
        UiComponentCategory::Visual,
        "revision-widget",
        UiDefaultNodeTemplate::native("RevisionWidget"),
    ));

    assert_eq!(registry.revision(), 0);
    assert_eq!(registry.register(descriptor.clone()), Ok(true));
    let first_revision = registry.revision();
    assert!(first_revision > 0);
    assert_eq!(registry.register(descriptor.clone()), Ok(false));
    assert_eq!(registry.revision(), first_revision);

    let changed = UiComponentDescriptor::new(
        "RevisionWidget",
        "Changed Revision Widget",
        UiComponentCategory::Visual,
        "revision-widget",
    )
    .default_node_template(UiDefaultNodeTemplate::native("RevisionWidget"))
    .palette(UiPaletteMetadata::new(
        "Changed Revision Widget",
        UiComponentCategory::Visual,
        "revision-widget",
        UiDefaultNodeTemplate::native("RevisionWidget"),
    ));
    assert_eq!(registry.register(changed), Ok(true));
    assert!(registry.revision() > first_revision);
}

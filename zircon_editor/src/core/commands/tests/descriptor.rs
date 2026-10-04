use crate::core::editor_operation::EditorOperationPath;

use crate::core::commands::EditorCommandRegistry;

use super::EditorCommandDescriptor;

#[test]
fn headless_commandlet_route_is_canonical_descriptor_metadata() {
    let registry = EditorCommandRegistry::default_workbench();
    let descriptor = registry
        .command("asset.migration.migrate_assets")
        .expect("the built-in migration command is registered");

    assert_eq!(
        descriptor
            .headless_commandlet_route()
            .map(crate::core::editor_operation::EditorOperationPath::as_str),
        Some("commandlet.route.migrate_assets")
    );
}

#[test]
fn command_enablement_does_not_materialize_an_effective_when_clause() {
    let source = include_str!("../descriptor.rs");
    let allocating_eval = ["self", ".effective_when().eval(context)"].concat();
    assert!(!source.contains(&allocating_eval));
}

#[test]
fn command_presentation_is_derived_from_stable_localization_keys() {
    let descriptor = EditorCommandDescriptor::operation(
        EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap(),
    );

    assert_eq!(
        descriptor.presentation().label_key(),
        "command.weather.cloud_layer.refresh.label"
    );
    assert_eq!(
        descriptor.presentation().description_key(),
        "command.weather.cloud_layer.refresh.description"
    );
}

#[test]
fn descriptor_production_shape_has_no_literal_presentation_fields() {
    let production = include_str!("../descriptor.rs")
        .split_once("#[cfg(test)]")
        .expect("descriptor tests should remain below production code")
        .0;

    assert!(!production.contains("display_name:"));
    assert!(!production.contains("description: String"));
    assert!(!production.contains("menu_path: Option<String>"));
}

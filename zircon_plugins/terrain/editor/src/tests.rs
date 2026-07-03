use super::*;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;

#[test]
fn terrain_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = zircon_editor::core::editor_extension::EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("terrain authoring registration");
    let operation =
        EditorOperationPath::parse("terrain.authoring.import_heightfield").expect("operation path");
    let descriptor = registry
        .operations()
        .descriptor(&operation)
        .expect("import heightfield operation registered");

    assert_eq!(
        descriptor.menu_path(),
        Some("Plugins/Terrain/Import Heightfield")
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("terrain.import_heightfield.v1")
    );
    assert!(registry.menu_items().iter().any(|item| {
        item.path() == "Plugins/Terrain/Import Heightfield" && item.operation() == &operation
    }));
}

#[test]
fn terrain_heightfield_import_accepts_supported_extensions_and_matching_samples() {
    let diagnostics = validate_heightfield_import(&TerrainHeightfieldImportRequest {
        width: 4,
        height: 4,
        sample_count: Some(16),
        source_extension: ".r16".to_string(),
    });

    assert!(diagnostics.is_empty());
    assert_eq!(
        terrain_import_output_kind("png"),
        Some("terrain.heightfield")
    );
}

#[test]
fn terrain_import_plan_selects_heightfield_or_layer_stack_output() {
    let request = TerrainHeightfieldImportRequest {
        width: 8,
        height: 4,
        sample_count: Some(32),
        source_extension: ".PNG".to_string(),
    };

    let heightfield = plan_terrain_import(TerrainImportKind::Heightfield, &request)
        .expect("heightfield import request is valid");
    let layer_stack = plan_terrain_import(TerrainImportKind::LayerStack, &request)
        .expect("layer stack import request is valid");

    assert_eq!(heightfield.normalized_extension, "png");
    assert_eq!(heightfield.output_kind, "terrain.heightfield");
    assert_eq!(layer_stack.output_kind, "terrain.layer_stack");
    assert_eq!(heightfield.expected_sample_count, 32);
}

#[test]
fn terrain_heightfield_import_reports_invalid_dimensions_extension_and_sample_count() {
    let mut diagnostics = validate_heightfield_import(&TerrainHeightfieldImportRequest {
        width: 0,
        height: 4,
        sample_count: None,
        source_extension: "exr".to_string(),
    });
    diagnostics.extend(validate_heightfield_import(
        &TerrainHeightfieldImportRequest {
            width: 2,
            height: 4,
            sample_count: Some(5),
            source_extension: "raw".to_string(),
        },
    ));

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("dimensions must be greater")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("extension `exr` is not supported")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("expected 8 samples")));
}

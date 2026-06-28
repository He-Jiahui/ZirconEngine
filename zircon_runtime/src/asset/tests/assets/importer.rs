use std::fs;
use std::path::Path;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    importer_with_first_wave_plugin_fixtures, sample_animation_sequence_asset,
    sample_physics_material_asset, write_checker_png, write_default_animation_sequence,
    write_default_physics_material,
};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporter,
    AssetImporterCapabilityStatus, AssetImporterDescriptor, AssetImporterRegistry,
    AssetImporterRegistryError, AssetUri, DataAssetFormat, DiagnosticOnlyAssetImporter,
    FunctionAssetImporter, ImportedAsset, MeshVertex, ModelAsset, ModelPrimitiveAsset,
};
use crate::core::math::{Vec2, Vec3};

mod builtin_data;
mod physics_animation;
mod registry_errors;
mod registry_priority;
mod shader_model;
mod structure;
mod typed_toml_ui;

fn valid_wgsl() -> &'static str {
    r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.4, 0.2, 1.0);
}
"#
}

fn minimal_zui_component_toml() -> &'static str {
    r#"
[asset]
kind = "component"
id = "runtime.ui.hud_overlay"
version = 2
display_name = "Runtime HUD Overlay"

[components.HudOverlay]
root = "root"

[nodes.root]
component = "Text"
control_id = "HudRoot"
props = { text = "HUD" }
"#
}

fn test_data_outcome(
    context: &AssetImportContext,
    winner: &'static str,
) -> Result<AssetImportOutcome, crate::asset::AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(crate::asset::DataAsset {
            uri: context.uri.clone(),
            format: crate::asset::DataAssetFormat::Json,
            text: String::from_utf8_lossy(&context.source_bytes).into_owned(),
            canonical_json: serde_json::json!({ "winner": winner }),
        }),
    ))
}

fn assert_cooked_virtual_geometry(primitive: &ModelPrimitiveAsset, source_hint: &str) {
    let virtual_geometry = primitive
        .virtual_geometry
        .as_ref()
        .expect("imported model primitive should carry cooked virtual geometry");
    assert!(!virtual_geometry.hierarchy_buffer.is_empty());
    assert!(!virtual_geometry.cluster_headers.is_empty());
    assert!(!virtual_geometry.cluster_page_headers.is_empty());
    assert!(!virtual_geometry.cluster_page_data.is_empty());
    assert!(!virtual_geometry.root_page_table.is_empty());
    assert_eq!(
        virtual_geometry.debug.source_hint.as_deref(),
        Some(source_hint)
    );
}

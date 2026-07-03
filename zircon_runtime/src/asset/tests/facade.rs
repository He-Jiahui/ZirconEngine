use std::time::Duration;

use crate::asset::{
    AlphaMode, AssetDependencyReadiness, AssetEvent, AssetLoadState, AssetLoadStates,
    AssetReference, AssetUri, Assets, DependencyLoadState, Handle, MaterialAsset, MeshAsset,
    ProjectAssetManager, RecursiveDependencyLoadState, ShaderAsset, ShaderEntryPointAsset,
    ShaderSourceLanguage, TextureAsset, UiLayoutAsset, UiV2ViewAsset,
};
use crate::core::framework::render::ShaderAssetKind;
use crate::core::resource::{
    ResourceDiagnostic, ResourceHandle, ResourceId, ResourceKind, ResourceManager, ResourceRecord,
    ResourceState, TextureMarker, UntypedResourceHandle,
};

mod dependency_failures;
mod failure_reason;
mod handle_events;
mod handle_lifecycle;
mod hot_reload;
mod load_state_roots;
mod project_facade;
mod recursive_dependencies;

fn locator(value: &str) -> AssetUri {
    AssetUri::parse(value).expect("valid asset locator")
}

fn record(locator_text: &str, kind: ResourceKind) -> ResourceRecord {
    let locator = locator(locator_text);
    ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator)
}

fn texture_asset(uri: &str) -> TextureAsset {
    TextureAsset::new_rgba8(locator(uri), 1, 1, vec![255, 0, 0, 255])
}

fn shader_asset(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() {}".to_string(),
        wgsl_source: "@fragment fn fs_main() {}".to_string(),
        import_path: None,
        entry_points: vec![ShaderEntryPointAsset {
            name: "fs_main".to_string(),
            stage: "fragment".to_string(),
        }],
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        options: Vec::new(),
        texture_slots: Vec::new(),
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn material_asset(shader_uri: &str) -> MaterialAsset {
    MaterialAsset {
        name: Some("Grid".to_string()),
        shader: AssetReference::from_locator(locator(shader_uri)),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.8, 0.8, 0.8, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn diagnostic_messages(diagnostics: &[ResourceDiagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

fn dependency_row(rows: &[AssetDependencyReadiness], id: ResourceId) -> &AssetDependencyReadiness {
    rows.iter()
        .find(|row| row.id == id)
        .expect("dependency row")
}

fn ui_v2_view_asset() -> UiV2ViewAsset {
    UiV2ViewAsset::from_toml_str(
        r#"
[asset]
kind = "view"
id = "runtime.ui.panel"
version = 2

[root]
node = "root"

[nodes.root]
component = "Text"
control_id = "PanelRoot"
props = { text = "Panel" }
"#,
    )
    .expect("valid ui v2 view asset")
}

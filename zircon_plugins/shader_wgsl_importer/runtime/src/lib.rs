use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, ShaderAsset,
    ShaderEntryPointAsset, ShaderSourceLanguage,
};
use zircon_runtime::core::framework::render::ShaderAssetKind;

mod capability;
mod plugin;

pub use capability::{
    IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
    SHADER_WGSL_IMPORTER_DECLARATION,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    ShaderWgslImporterRuntimePlugin, SHADER_WGSL_IMPORTER_DIST_CRATE_NAME,
    SHADER_WGSL_IMPORTER_DIST_RUNTIME_ENTRY,
};

pub fn import_wgsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_text()?;
    let module = wgsl::parse_str(&source).map_err(|error| {
        AssetImportError::ShaderValidation(format!(
            "{}: {}",
            context.uri,
            error.emit_to_string(&source)
        ))
    })?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator
        .validate(&module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{}: {error}", context.uri)))?;
    let entry_points = module
        .entry_points
        .iter()
        .map(|entry| ShaderEntryPointAsset {
            name: entry.name.clone(),
            stage: format!("{:?}", entry.stage).to_ascii_lowercase(),
        })
        .collect();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Shader(ShaderAsset {
            uri: context.uri.clone(),
            kind: ShaderAssetKind::Surface,
            source_language: ShaderSourceLanguage::Wgsl,
            source: source.clone(),
            wgsl_source: source,
            import_path: None,
            entry_points,
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
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::framework::project::ExportPackagingStrategy;

    #[test]
    fn package_declares_wgsl_importer() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"wgsl".to_string())));
    }

    #[test]
    fn declaration_projects_wgsl_package_metadata() {
        let descriptor = runtime_plugin_descriptor();
        let manifest = package_manifest();

        assert_eq!(
            descriptor.package_id(),
            SHADER_WGSL_IMPORTER_DECLARATION.id()
        );
        assert_eq!(
            descriptor.category(),
            SHADER_WGSL_IMPORTER_DECLARATION.category()
        );
        assert_eq!(
            descriptor.target_modes(),
            SHADER_WGSL_IMPORTER_DECLARATION.target_modes()
        );
        assert_eq!(
            descriptor.capabilities(),
            runtime_capabilities()
                .iter()
                .map(|capability| capability.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            manifest.supported_platforms.as_slice(),
            SHADER_WGSL_IMPORTER_DECLARATION.supported_platforms()
        );
        assert_eq!(
            manifest.default_packaging.as_slice(),
            SHADER_WGSL_IMPORTER_DECLARATION.default_packaging()
        );
    }

    #[test]
    fn package_manifest_declares_wgsl_importer_dist_contract() {
        let manifest = package_manifest();
        let distribution = manifest
            .distribution
            .as_ref()
            .expect("WGSL importer package exposes dist metadata");

        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic));
        assert_eq!(distribution.forms, vec!["dist"]);
        assert_eq!(
            distribution.default_packaging,
            vec![ExportPackagingStrategy::NativeDynamic]
        );
        assert_eq!(distribution.abi_version, Some(3));
        assert_eq!(
            distribution.dist_crate,
            SHADER_WGSL_IMPORTER_DIST_CRATE_NAME
        );
        assert_eq!(
            distribution.runtime_entry,
            SHADER_WGSL_IMPORTER_DIST_RUNTIME_ENTRY
        );

        let dist_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "shader_wgsl_importer.dist")
            .expect("WGSL importer package includes native dist module");
        assert_eq!(
            dist_module.kind,
            zircon_runtime::plugin::PluginModuleKind::Native
        );
        assert_eq!(dist_module.crate_name, SHADER_WGSL_IMPORTER_DIST_CRATE_NAME);
        assert!(dist_module
            .capabilities
            .contains(&IMPORTER_CAPABILITY.to_string()));
    }

    #[test]
    fn registration_contributes_module_and_importer() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert!(report
            .extensions
            .asset_importers()
            .descriptors()
            .iter()
            .any(|importer| importer.id == "shader_wgsl_importer.wgsl"));
    }

    #[test]
    fn wgsl_importer_validates_shader_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("main.wgsl"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "main.wgsl".into(),
            zircon_runtime::asset::AssetUri::parse("res://shaders/main.wgsl").unwrap(),
            valid_wgsl().as_bytes().to_vec(),
            Default::default(),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome.root_entry().expect("root shader asset entry").asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Shader(shader) => {
                assert_eq!(shader.source_language, ShaderSourceLanguage::Wgsl);
                assert_eq!(shader.entry_points.len(), 2);
                assert!(shader.wgsl_source.contains("vs_main"));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

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
}

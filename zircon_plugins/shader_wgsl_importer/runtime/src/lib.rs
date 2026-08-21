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
    IMPORTER_CAPABILITY, MODULE_NAME, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITY,
    RUNTIME_CRATE_NAME, SHADER_WGSL_IMPORTER_DECLARATION,
};
pub use plugin::{
    SHADER_WGSL_IMPORTER_DIST_CRATE_NAME, SHADER_WGSL_IMPORTER_DIST_RUNTIME_ENTRY,
    ShaderWgslImporterRuntimePlugin, asset_importer_descriptors, dist_module_manifest,
    module_descriptor, package_manifest, plugin_registration, runtime_capabilities,
    runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor, runtime_selection,
    supported_platforms, supported_targets,
};

pub fn import_wgsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_str()?;
    let module = wgsl::parse_str(source).map_err(|error| {
        AssetImportError::ShaderValidation(format!(
            "{}: {}",
            context.uri,
            error.emit_to_string(source)
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
    let source = source.to_owned();
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
        assert!(
            manifest
                .capabilities
                .contains(&RUNTIME_CAPABILITY.to_string())
        );
        assert!(
            manifest
                .asset_importers
                .iter()
                .any(|importer| importer.source_extensions.contains(&"wgsl".to_string()))
        );
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

        assert!(
            manifest
                .default_packaging
                .contains(&ExportPackagingStrategy::NativeDynamic)
        );
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
        assert!(
            dist_module
                .capabilities
                .contains(&IMPORTER_CAPABILITY.to_string())
        );
    }

    #[test]
    fn registration_contributes_module_and_importer() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(
            report
                .extensions
                .modules()
                .iter()
                .any(|module| module.name == MODULE_NAME)
        );
        assert!(
            report
                .extensions
                .asset_importers()
                .descriptors()
                .iter()
                .any(|importer| importer.id == "shader_wgsl_importer.wgsl")
        );
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

    #[test]
    fn wgsl_importer_rejects_invalid_borrowed_source() {
        let context = zircon_runtime::asset::AssetImportContext::new(
            "broken.wgsl".into(),
            zircon_runtime::asset::AssetUri::parse("res://shaders/broken.wgsl").unwrap(),
            b"@".to_vec(),
            Default::default(),
        );

        let error = import_wgsl(&context).unwrap_err();

        assert!(error.to_string().contains("res://shaders/broken.wgsl"));
    }

    #[test]
    #[ignore = "release performance gate; run through the managed Plugins05 validator"]
    fn wgsl_borrowed_source_release_gate() {
        use std::hint::black_box;
        use std::time::Instant;

        const SOURCE_BYTES: usize = 1_048_576;
        const ITERATIONS: usize = 32;
        const SAMPLE_PAIRS: usize = 21;
        const TARGET_P95_PERCENT: u128 = 85;

        let context = zircon_runtime::asset::AssetImportContext::new(
            "invalid-large.wgsl".into(),
            zircon_runtime::asset::AssetUri::parse("res://shaders/invalid-large.wgsl").unwrap(),
            vec![b'@'; SOURCE_BYTES],
            Default::default(),
        );
        let mut owned_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut borrowed_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let measure_owned = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    let source = context.source_text().unwrap();
                    black_box(wgsl::parse_str(black_box(source.as_str())).is_err());
                }
                started.elapsed().as_nanos()
            };
            let measure_borrowed = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    let source = context.source_str().unwrap();
                    black_box(wgsl::parse_str(black_box(source)).is_err());
                }
                started.elapsed().as_nanos()
            };
            if pair_index % 2 == 0 {
                owned_samples_ns.push(measure_owned());
                borrowed_samples_ns.push(measure_borrowed());
            } else {
                borrowed_samples_ns.push(measure_borrowed());
                owned_samples_ns.push(measure_owned());
            }
        }

        let owned_p95_ns = nearest_rank_percentile(&owned_samples_ns, 95);
        let borrowed_p95_ns = nearest_rank_percentile(&borrowed_samples_ns, 95);
        assert!(
            borrowed_p95_ns * 100 <= owned_p95_ns * TARGET_P95_PERCENT,
            "borrowed-source P95 {borrowed_p95_ns}ns exceeded {TARGET_P95_PERCENT}% of owned-source P95 {owned_p95_ns}ns"
        );
        let owned_samples_csv = join_samples(&owned_samples_ns);
        let borrowed_samples_csv = join_samples(&borrowed_samples_ns);
        let owned_clone_bytes_per_sample = SOURCE_BYTES * ITERATIONS;
        println!(
            "PERF-MVP-PLUGINS05-BORROWED-SHADER-SOURCE source_bytes={SOURCE_BYTES} iterations_per_sample={ITERATIONS} sample_pairs={SAMPLE_PAIRS} order=alternating_owned_first_even percentile_method=nearest_rank owned_clone_bytes_per_sample={owned_clone_bytes_per_sample} borrowed_clone_bytes_per_sample=0 clone_byte_reduction_percent=100 owned_p95_ns={owned_p95_ns} borrowed_p95_ns={borrowed_p95_ns} target_p95_percent={TARGET_P95_PERCENT} owned_samples_ns={owned_samples_csv} borrowed_samples_ns={borrowed_samples_csv}"
        );
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
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

use naga::valid::{Capabilities, ValidationFlags, Validator};
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, ShaderAsset,
    ShaderEntryPointAsset, ShaderSourceLanguage,
};
use zircon_runtime::core::framework::render::ShaderAssetKind;

mod capability;
mod plugin;

pub use capability::{
    IMPORTER_FAMILY, MODULE_NAME, NAGA_IMPORTER_CAPABILITY, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST,
    PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME, SHADER_ASSET_IMPORTER_DECLARATION,
    WGSL_IMPORTER_CAPABILITY,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    ShaderAssetImporterRuntimePlugin, SHADER_ASSET_IMPORTER_DIST_CRATE_NAME,
    SHADER_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
};

pub fn import_shader(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "wgsl" => import_wgsl(context),
        "glsl" | "vert" | "frag" | "comp" | "vs" | "fs" | "cs" => import_glsl(context),
        "spv" => import_spirv(context),
        _ => Err(AssetImportError::UnsupportedFormat(format!(
            "shader importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

fn import_wgsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_str()?;
    let module = naga::front::wgsl::parse_str(source).map_err(|error| {
        AssetImportError::ShaderValidation(format!(
            "{}: {}",
            context.uri,
            error.emit_to_string(source)
        ))
    })?;
    module_to_shader_asset(context, ShaderSourceLanguage::Wgsl, source, module)
}

fn import_glsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_str()?;
    let stage = infer_shader_stage(context)?;
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&naga::front::glsl::Options::from(stage), source)
        .map_err(|error| {
            AssetImportError::ShaderValidation(format!(
                "{}: {}",
                context.uri,
                error.emit_to_string(source)
            ))
        })?;
    let info = validate_naga_module(context, &module)?;
    let wgsl_source = module_to_wgsl(context, &module, &info)?;
    shader_outcome(
        context,
        ShaderSourceLanguage::Glsl,
        source.to_owned(),
        wgsl_source,
        shader_entry_points(&module),
    )
}

fn import_spirv(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let module = naga::front::spv::parse_u8_slice(
        &context.source_bytes,
        &naga::front::spv::Options::default(),
    )
    .map_err(|error| {
        AssetImportError::ShaderValidation(format!("{}: spir-v parse failed: {error}", context.uri))
    })?;
    let info = validate_naga_module(context, &module)?;
    let wgsl_source = module_to_wgsl(context, &module, &info)?;
    shader_outcome(
        context,
        ShaderSourceLanguage::SpirV,
        hex_encode(&context.source_bytes),
        wgsl_source,
        shader_entry_points(&module),
    )
}

fn module_to_shader_asset(
    context: &AssetImportContext,
    source_language: ShaderSourceLanguage,
    source: &str,
    module: naga::Module,
) -> Result<AssetImportOutcome, AssetImportError> {
    validate_naga_module(context, &module)?;
    let source = source.to_owned();
    shader_outcome(
        context,
        source_language,
        source.clone(),
        source,
        shader_entry_points(&module),
    )
}

fn shader_outcome(
    context: &AssetImportContext,
    source_language: ShaderSourceLanguage,
    source: String,
    wgsl_source: String,
    entry_points: Vec<ShaderEntryPointAsset>,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Shader(ShaderAsset {
            uri: context.uri.clone(),
            kind: ShaderAssetKind::Module,
            source_language,
            source,
            wgsl_source,
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

fn validate_naga_module(
    context: &AssetImportContext,
    module: &naga::Module,
) -> Result<naga::valid::ModuleInfo, AssetImportError> {
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator
        .validate(module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{}: {error}", context.uri)))
}

fn module_to_wgsl(
    context: &AssetImportContext,
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
) -> Result<String, AssetImportError> {
    naga::back::wgsl::write_string(module, info, naga::back::wgsl::WriterFlags::empty()).map_err(
        |error| {
            AssetImportError::ShaderValidation(format!(
                "{}: wgsl emission failed: {error}",
                context.uri
            ))
        },
    )
}

fn infer_shader_stage(context: &AssetImportContext) -> Result<naga::ShaderStage, AssetImportError> {
    if let Some(stage) = context
        .import_settings
        .get("shader_stage")
        .and_then(|value| value.as_str())
    {
        return parse_shader_stage(stage);
    }

    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    let extension_stage = match extension.to_ascii_lowercase().as_str() {
        "vert" | "vs" => Some(naga::ShaderStage::Vertex),
        "frag" | "fs" => Some(naga::ShaderStage::Fragment),
        "comp" | "cs" => Some(naga::ShaderStage::Compute),
        _ => None,
    };
    if let Some(stage) = extension_stage {
        return Ok(stage);
    }

    let stem_hint = context
        .source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.rsplit('.').next())
        .unwrap_or_default();
    if stem_hint.is_empty() {
        Ok(naga::ShaderStage::Vertex)
    } else {
        parse_shader_stage(stem_hint).or(Ok(naga::ShaderStage::Vertex))
    }
}

fn parse_shader_stage(stage: &str) -> Result<naga::ShaderStage, AssetImportError> {
    match stage.trim().to_ascii_lowercase().as_str() {
        "vertex" | "vert" | "vs" => Ok(naga::ShaderStage::Vertex),
        "fragment" | "frag" | "fs" => Ok(naga::ShaderStage::Fragment),
        "compute" | "comp" | "cs" => Ok(naga::ShaderStage::Compute),
        other => Err(AssetImportError::Parse(format!(
            "unsupported shader stage `{other}`"
        ))),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn shader_entry_points(module: &naga::Module) -> Vec<ShaderEntryPointAsset> {
    module
        .entry_points
        .iter()
        .map(|entry| ShaderEntryPointAsset {
            name: entry.name.clone(),
            stage: shader_stage_name(&entry.stage).to_owned(),
        })
        .collect()
}

fn shader_stage_name(stage: &naga::ShaderStage) -> &'static str {
    match stage {
        naga::ShaderStage::Vertex => "vertex",
        naga::ShaderStage::Task => "task",
        naga::ShaderStage::Mesh => "mesh",
        naga::ShaderStage::Fragment => "fragment",
        naga::ShaderStage::Compute => "compute",
        naga::ShaderStage::RayGeneration => "raygeneration",
        naga::ShaderStage::Miss => "miss",
        naga::ShaderStage::AnyHit => "anyhit",
        naga::ShaderStage::ClosestHit => "closesthit",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_shader_importer_capabilities() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"hlsl".to_string())));
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(!manifest
            .capabilities
            .contains(&WGSL_IMPORTER_CAPABILITY.to_string()));
        assert!(manifest
            .capabilities
            .contains(&NAGA_IMPORTER_CAPABILITY.to_string()));
    }

    #[test]
    fn shader_asset_importer_package_manifest_declares_dist_contract() {
        let manifest = package_manifest();
        let distribution = manifest
            .distribution
            .as_ref()
            .expect("shader importer package exposes dist metadata");

        assert!(manifest.default_packaging.contains(
            &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
        ));
        assert_eq!(distribution.forms, vec!["dist"]);
        assert_eq!(
            distribution.default_packaging,
            vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
        );
        assert_eq!(distribution.abi_version, Some(3));
        assert_eq!(
            distribution.dist_crate,
            SHADER_ASSET_IMPORTER_DIST_CRATE_NAME
        );
        assert_eq!(
            distribution.runtime_entry,
            SHADER_ASSET_IMPORTER_DIST_RUNTIME_ENTRY
        );

        let dist_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "asset_importer.shader.dist")
            .expect("shader importer package includes native dist module");
        assert_eq!(
            dist_module.kind,
            zircon_runtime::plugin::PluginModuleKind::Native
        );
        assert_eq!(
            dist_module.crate_name,
            SHADER_ASSET_IMPORTER_DIST_CRATE_NAME
        );
        assert!(dist_module.target_modes.contains(
            &zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
        ));
        assert!(dist_module
            .target_modes
            .contains(&zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost));
        assert!(dist_module
            .capabilities
            .contains(&NAGA_IMPORTER_CAPABILITY.to_string()));
        assert!(!dist_module
            .capabilities
            .contains(&WGSL_IMPORTER_CAPABILITY.to_string()));
    }

    #[test]
    fn registration_contributes_module_and_shader_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 3);
    }

    #[test]
    fn wgsl_importer_validates_shader_asset() {
        let asset = import_fixture("main.wgsl", valid_wgsl(), Default::default());

        match asset {
            ImportedAsset::Shader(shader) => {
                assert_eq!(shader.kind, ShaderAssetKind::Module);
                assert_eq!(shader.source_language, ShaderSourceLanguage::Wgsl);
                assert_eq!(shader.entry_points.len(), 2);
                assert!(shader.wgsl_source.contains("vs_main"));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn glsl_importer_emits_wgsl_shader_asset() {
        let asset = import_fixture("main.vert", valid_glsl_vertex(), Default::default());

        match asset {
            ImportedAsset::Shader(shader) => {
                assert_eq!(shader.kind, ShaderAssetKind::Module);
                assert_eq!(shader.source_language, ShaderSourceLanguage::Glsl);
                assert_eq!(shader.entry_points.len(), 1);
                assert_eq!(shader.entry_points[0].stage, "vertex");
                assert!(shader.wgsl_source.contains("@vertex"));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn glsl_importer_uses_shader_stage_import_setting() {
        let mut settings = toml::Table::new();
        settings.insert("shader_stage".to_string(), "fragment".into());
        let asset = import_fixture("main.glsl", valid_glsl_fragment(), settings);

        match asset {
            ImportedAsset::Shader(shader) => {
                assert_eq!(shader.kind, ShaderAssetKind::Module);
                assert_eq!(shader.source_language, ShaderSourceLanguage::Glsl);
                assert_eq!(shader.entry_points[0].stage, "fragment");
                assert!(shader.wgsl_source.contains("@fragment"));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn invalid_glsl_returns_shader_error() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("broken.vert"))
            .unwrap();
        let context = context_for(
            "broken.vert",
            "#version 450\nvoid main( {",
            Default::default(),
        );

        let error = importer.import(&context).unwrap_err();

        assert!(error.to_string().contains("wgsl validation failed"));
    }

    #[test]
    fn plugins07_importer_hotpath_shader_stage_names_preserve_all_naga_stages() {
        assert_eq!(shader_stage_name(&naga::ShaderStage::Vertex), "vertex");
        assert_eq!(shader_stage_name(&naga::ShaderStage::Task), "task");
        assert_eq!(shader_stage_name(&naga::ShaderStage::Mesh), "mesh");
        assert_eq!(shader_stage_name(&naga::ShaderStage::Fragment), "fragment");
        assert_eq!(shader_stage_name(&naga::ShaderStage::Compute), "compute");
        assert_eq!(
            shader_stage_name(&naga::ShaderStage::RayGeneration),
            "raygeneration"
        );
        assert_eq!(shader_stage_name(&naga::ShaderStage::Miss), "miss");
        assert_eq!(shader_stage_name(&naga::ShaderStage::AnyHit), "anyhit");
        assert_eq!(
            shader_stage_name(&naga::ShaderStage::ClosestHit),
            "closesthit"
        );
    }

    #[test]
    #[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
    fn plugins07_importer_hotpath_release_static_shader_stage_name_p95_gate() {
        use std::hint::black_box;
        use std::time::Instant;

        const SAMPLE_PAIRS: usize = 21;
        const STAGE_NAMES: usize = 262_144;
        const THRESHOLD_PERCENT: u128 = 20;
        let stages = [
            naga::ShaderStage::Vertex,
            naga::ShaderStage::Task,
            naga::ShaderStage::Mesh,
            naga::ShaderStage::Fragment,
            naga::ShaderStage::Compute,
            naga::ShaderStage::RayGeneration,
            naga::ShaderStage::Miss,
            naga::ShaderStage::AnyHit,
            naga::ShaderStage::ClosestHit,
        ];
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let legacy = || {
                let started = Instant::now();
                let mut bytes = 0_usize;
                for index in 0..STAGE_NAMES {
                    let stage = black_box(&stages[index % stages.len()]);
                    let name = format!("{stage:?}").to_ascii_lowercase();
                    bytes += black_box(name.as_str()).len();
                }
                black_box(bytes);
                started.elapsed().as_nanos()
            };
            let optimized = || {
                let started = Instant::now();
                let mut bytes = 0_usize;
                for index in 0..STAGE_NAMES {
                    let stage = black_box(&stages[index % stages.len()]);
                    let name = shader_stage_name(stage).to_owned();
                    bytes += black_box(name.as_str()).len();
                }
                black_box(bytes);
                started.elapsed().as_nanos()
            };
            if pair % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }

        emit_shader_stage_performance_gate(
            &legacy_samples,
            &optimized_samples,
            THRESHOLD_PERCENT,
            &format!(
                "stage_names_per_sample={STAGE_NAMES} legacy_stage_string_allocations_per_sample={} optimized_stage_string_allocations_per_sample={STAGE_NAMES}",
                STAGE_NAMES * 2
            ),
        );
    }

    fn emit_shader_stage_performance_gate(
        legacy_samples: &[u128],
        optimized_samples: &[u128],
        threshold_percent: u128,
        workload: &str,
    ) {
        let legacy_p95 = nearest_rank_shader_stage_p95(legacy_samples);
        let optimized_p95 = nearest_rank_shader_stage_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_shader_static_stage_name sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
            shader_stage_samples_csv(legacy_samples),
            shader_stage_samples_csv(optimized_samples),
        );
        assert!(
            improvement_percent >= threshold_percent,
            "static shader stage names must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
        );
    }

    fn nearest_rank_shader_stage_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn shader_stage_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn import_fixture(path: &str, source: &str, settings: toml::Table) -> ImportedAsset {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new(path))
            .unwrap();
        importer
            .import(&context_for(path, source, settings))
            .unwrap()
            .root_entry()
            .map(|entry| entry.asset.clone())
            .expect("shader importer root asset")
    }

    fn context_for(path: &str, source: &str, settings: toml::Table) -> AssetImportContext {
        let file_name = path.replace('\\', "/");
        let uri = format!("res://shaders/{file_name}");
        AssetImportContext::new(
            path.into(),
            zircon_runtime::asset::AssetUri::parse(&uri).unwrap(),
            source.as_bytes().to_vec(),
            settings,
        )
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

    fn valid_glsl_vertex() -> &'static str {
        r#"#version 450
layout(location = 0) in vec3 a_position;
void main() {
    gl_Position = vec4(a_position, 1.0);
}
"#
    }

    fn valid_glsl_fragment() -> &'static str {
        r#"#version 450
layout(location = 0) out vec4 o_color;
void main() {
    o_color = vec4(1.0, 0.4, 0.2, 1.0);
}
"#
    }
}

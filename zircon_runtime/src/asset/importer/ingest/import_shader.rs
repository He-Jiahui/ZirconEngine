use super::validate_wgsl::{validate_naga_module, validate_wgsl};
use crate::asset::assets::{
    ImportedAsset, ShaderAsset, ShaderEntryPointAsset, ShaderSourceLanguage,
};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};
use crate::core::framework::render::ShaderAssetKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShaderSourceKind {
    Wgsl,
    Glsl,
    SpirV,
}

impl ShaderSourceKind {
    fn from_extension(extension: &str) -> Option<Self> {
        match extension.len() {
            2 if ["vs", "fs", "cs"]
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate)) =>
            {
                Some(Self::Glsl)
            }
            3 if extension.eq_ignore_ascii_case("spv") => Some(Self::SpirV),
            4 if extension.eq_ignore_ascii_case("wgsl") => Some(Self::Wgsl),
            4 if ["glsl", "vert", "frag", "comp"]
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate)) =>
            {
                Some(Self::Glsl)
            }
            _ => None,
        }
    }
}

pub(crate) fn import_shader(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    match ShaderSourceKind::from_extension(extension) {
        Some(ShaderSourceKind::Wgsl) => import_wgsl(context),
        Some(ShaderSourceKind::Glsl) => import_glsl(context),
        Some(ShaderSourceKind::SpirV) => import_spirv(context),
        None => Err(AssetImportError::UnsupportedFormat(format!(
            "shader importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

fn import_wgsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_text()?;
    let (module, _info) = validate_wgsl(&context.uri, &source)?;
    let entry_points = shader_entry_points(&module);
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Shader(ShaderAsset {
            uri: context.uri.clone(),
            kind: ShaderAssetKind::Module,
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

fn import_glsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_text()?;
    let stage = infer_shader_stage(context)?;
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&naga::front::glsl::Options::from(stage), &source)
        .map_err(|error| {
            AssetImportError::ShaderValidation(format!(
                "{}: {}",
                context.uri,
                error.emit_to_string(&source)
            ))
        })?;
    module_to_shader_asset(context, ShaderSourceLanguage::Glsl, source, module)
}

fn import_spirv(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let module = naga::front::spv::parse_u8_slice(
        &context.source_bytes,
        &naga::front::spv::Options::default(),
    )
    .map_err(|error| {
        AssetImportError::ShaderValidation(format!("{}: spir-v parse failed: {error}", context.uri))
    })?;
    module_to_shader_asset(
        context,
        ShaderSourceLanguage::SpirV,
        hex_encode(&context.source_bytes),
        module,
    )
}

fn module_to_shader_asset(
    context: &AssetImportContext,
    source_language: ShaderSourceLanguage,
    source: String,
    module: naga::Module,
) -> Result<AssetImportOutcome, AssetImportError> {
    let info = validate_naga_module(&context.uri, &module)?;
    let wgsl_source =
        naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
            .map_err(|error| {
                AssetImportError::ShaderValidation(format!(
                    "{}: wgsl emission failed: {error}",
                    context.uri
                ))
            })?;
    let entry_points = shader_entry_points(&module);
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
    let extension_stage = shader_stage_extension_hint(extension);
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
    let stage = stage.trim();
    if let Some(stage) = shader_stage_hint(stage) {
        return Ok(stage);
    }
    let normalized = stage.to_ascii_lowercase();
    Err(AssetImportError::Parse(format!(
        "unsupported shader stage `{normalized}`"
    )))
}

fn shader_stage_extension_hint(extension: &str) -> Option<naga::ShaderStage> {
    match extension.len() {
        2 if extension.eq_ignore_ascii_case("vs") => Some(naga::ShaderStage::Vertex),
        2 if extension.eq_ignore_ascii_case("fs") => Some(naga::ShaderStage::Fragment),
        2 if extension.eq_ignore_ascii_case("cs") => Some(naga::ShaderStage::Compute),
        4 if extension.eq_ignore_ascii_case("vert") => Some(naga::ShaderStage::Vertex),
        4 if extension.eq_ignore_ascii_case("frag") => Some(naga::ShaderStage::Fragment),
        4 if extension.eq_ignore_ascii_case("comp") => Some(naga::ShaderStage::Compute),
        _ => None,
    }
}

fn shader_stage_hint(stage: &str) -> Option<naga::ShaderStage> {
    let stage = stage.trim();
    match stage.len() {
        2 if stage.eq_ignore_ascii_case("vs") => Some(naga::ShaderStage::Vertex),
        2 if stage.eq_ignore_ascii_case("fs") => Some(naga::ShaderStage::Fragment),
        2 if stage.eq_ignore_ascii_case("cs") => Some(naga::ShaderStage::Compute),
        4 if stage.eq_ignore_ascii_case("vert") => Some(naga::ShaderStage::Vertex),
        4 if stage.eq_ignore_ascii_case("frag") => Some(naga::ShaderStage::Fragment),
        4 if stage.eq_ignore_ascii_case("comp") => Some(naga::ShaderStage::Compute),
        6 if stage.eq_ignore_ascii_case("vertex") => Some(naga::ShaderStage::Vertex),
        7 if stage.eq_ignore_ascii_case("compute") => Some(naga::ShaderStage::Compute),
        8 if stage.eq_ignore_ascii_case("fragment") => Some(naga::ShaderStage::Fragment),
        _ => None,
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

pub(super) fn shader_entry_points(module: &naga::Module) -> Vec<ShaderEntryPointAsset> {
    module
        .entry_points
        .iter()
        .map(|entry| ShaderEntryPointAsset {
            name: entry.name.clone(),
            stage: format!("{:?}", entry.stage).to_ascii_lowercase(),
        })
        .collect()
}

#[cfg(test)]
mod plugins07_builtin_shader_hotpath_tests {
    use std::{hint::black_box, time::Instant};

    use super::*;

    const SOURCE_EXTENSIONS: [&str; 10] = [
        "WGSL", "wgsl", "GLSL", "Vert", "FRAG", "comp", "VS", "fs", "Cs", "SPV",
    ];
    const STAGE_NAMES: [&str; 9] = [
        "Vertex", "VERT", "vs", "Fragment", "FRAG", "fs", "Compute", "COMP", "cs",
    ];

    #[test]
    fn raw_wgsl_imports_as_generic_shader_module() {
        let context = AssetImportContext::new(
            "module.wgsl".into(),
            crate::asset::AssetUri::parse("res://shaders/module.wgsl").unwrap(),
            b"fn helper() {}".to_vec(),
            Default::default(),
        );

        let outcome = import_wgsl(&context).unwrap();
        let ImportedAsset::Shader(shader) = &outcome.root_entry().unwrap().asset else {
            panic!("expected WGSL shader asset");
        };

        assert_eq!(shader.kind, ShaderAssetKind::Module);
        assert!(shader.entry_points.is_empty());
        assert!(shader.readiness_report().is_ready());
    }

    fn legacy_shader_source_kind(extension: &str) -> Option<ShaderSourceKind> {
        match extension.to_ascii_lowercase().as_str() {
            "wgsl" => Some(ShaderSourceKind::Wgsl),
            "glsl" | "vert" | "frag" | "comp" | "vs" | "fs" | "cs" => Some(ShaderSourceKind::Glsl),
            "spv" => Some(ShaderSourceKind::SpirV),
            _ => None,
        }
    }

    fn legacy_shader_stage_hint(stage: &str) -> Option<naga::ShaderStage> {
        match stage.trim().to_ascii_lowercase().as_str() {
            "vertex" | "vert" | "vs" => Some(naga::ShaderStage::Vertex),
            "fragment" | "frag" | "fs" => Some(naga::ShaderStage::Fragment),
            "compute" | "comp" | "cs" => Some(naga::ShaderStage::Compute),
            _ => None,
        }
    }

    #[test]
    fn plugins07_builtin_shader_hotpath_extension_dispatch_preserves_ascii_case_matching() {
        for extension in SOURCE_EXTENSIONS {
            assert_eq!(
                ShaderSourceKind::from_extension(extension),
                legacy_shader_source_kind(extension),
            );
        }
        assert_eq!(ShaderSourceKind::from_extension("metal"), None);
    }

    #[test]
    fn plugins07_builtin_shader_hotpath_stage_parse_preserves_aliases_and_diagnostics() {
        for stage in STAGE_NAMES {
            assert_eq!(shader_stage_hint(stage), legacy_shader_stage_hint(stage));
        }
        assert_eq!(shader_stage_hint("geometry"), None);
        let error = parse_shader_stage(" Geometry ")
            .expect_err("unknown shader stage must fail closed")
            .to_string();
        assert!(
            error.contains("unsupported shader stage `geometry`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    #[ignore = "release-only borrowed shader extension dispatch benchmark"]
    fn plugins07_builtin_shader_hotpath_release_borrowed_extension_dispatch_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 100_000;
        let (legacy_samples, optimized_samples) = alternating_samples(
            SAMPLE_PAIRS,
            || {
                measure(CHECKS_PER_SAMPLE, &SOURCE_EXTENSIONS, |extension| {
                    legacy_shader_source_kind(extension)
                })
            },
            || {
                measure(CHECKS_PER_SAMPLE, &SOURCE_EXTENSIONS, |extension| {
                    ShaderSourceKind::from_extension(extension)
                })
            },
        );
        report_and_assert(
            "plugins07_builtin_shader_extension_dispatch",
            SAMPLE_PAIRS,
            CHECKS_PER_SAMPLE,
            SOURCE_EXTENSIONS.len(),
            &legacy_samples,
            &optimized_samples,
        );
    }

    #[test]
    #[ignore = "release-only borrowed shader stage parse benchmark"]
    fn plugins07_builtin_shader_hotpath_release_borrowed_stage_parse_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 100_000;
        let (legacy_samples, optimized_samples) = alternating_samples(
            SAMPLE_PAIRS,
            || {
                measure(CHECKS_PER_SAMPLE, &STAGE_NAMES, |stage| {
                    legacy_shader_stage_hint(stage)
                })
            },
            || measure(CHECKS_PER_SAMPLE, &STAGE_NAMES, shader_stage_hint),
        );
        report_and_assert(
            "plugins07_builtin_shader_stage_parse",
            SAMPLE_PAIRS,
            CHECKS_PER_SAMPLE,
            STAGE_NAMES.len(),
            &legacy_samples,
            &optimized_samples,
        );
    }

    fn measure<T: Copy>(
        checks_per_sample: usize,
        values: &[&str],
        mut classify: impl FnMut(&str) -> Option<T>,
    ) -> u128 {
        let started = Instant::now();
        for check in 0..checks_per_sample {
            let value = black_box(values[check % values.len()]);
            black_box(classify(value));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn alternating_samples(
        sample_pairs: usize,
        mut legacy: impl FnMut() -> u128,
        mut optimized: impl FnMut() -> u128,
    ) -> (Vec<u128>, Vec<u128>) {
        for _ in 0..4 {
            black_box(legacy());
            black_box(optimized());
        }
        let mut legacy_samples = Vec::with_capacity(sample_pairs);
        let mut optimized_samples = Vec::with_capacity(sample_pairs);
        for pair in 0..sample_pairs {
            if pair % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn report_and_assert(
        name: &str,
        sample_pairs: usize,
        checks_per_sample: usize,
        variants: usize,
        legacy_samples: &[u128],
        optimized_samples: &[u128],
    ) {
        let legacy_p95_ns = percentile(legacy_samples, 95);
        let optimized_p95_ns = percentile(optimized_samples, 95);
        let improvement_percent = improvement_percent(legacy_p95_ns, optimized_p95_ns);
        println!(
            "PERF_RESULT {name} sample_pairs={sample_pairs} \
checks_per_sample={checks_per_sample} variants={variants} \
order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_owned_strings_per_sample={checks_per_sample} optimized_owned_strings_per_sample=0 \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
improvement_percent={improvement_percent} threshold_percent=50 \
legacy_ns={} optimized_ns={}",
            raw(legacy_samples),
            raw(optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
            "borrowed shader classification must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
        if optimized >= legacy {
            0
        } else {
            legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
        }
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;

use zircon_runtime::core::framework::render::{
    builtin_geometry_source_descriptors, GeometrySourceId, ShaderQualityTier,
    ShaderVariantPrewarmExecutionBudget, ShadingModelId, GEOMETRY_SOURCE_ID_MORPHED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
    GEOMETRY_SOURCE_ID_STATIC_MESH, GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START,
};

use super::error::{ShaderPrewarmArgsError, ShaderPrewarmArgsResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderPrewarmArgs {
    pub project_root: PathBuf,
    pub manifest: Option<PathBuf>,
    pub asset_roots: Vec<PathBuf>,
    pub quality_tiers: Vec<ShaderQualityTier>,
    pub geometry_sources: Vec<GeometrySourceId>,
    pub geometry_source_ids: BTreeMap<String, GeometrySourceId>,
    pub shading_model_ids: BTreeMap<String, ShadingModelId>,
    pub permutation_registries: Vec<PathBuf>,
    pub resource_registry: Option<PathBuf>,
    pub export_resource_registry: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub report: Option<PathBuf>,
    pub builtin_fallback: bool,
    pub validate_wgpu_modules: bool,
    pub validate_wgpu_pipelines: bool,
    pub execution_budget: ShaderVariantPrewarmExecutionBudget,
    pub pretty: bool,
}

pub fn parse(
    args: impl IntoIterator<Item = OsString>,
) -> ShaderPrewarmArgsResult<Option<ShaderPrewarmArgs>> {
    let mut project_root = PathBuf::from(".");
    let mut manifest = None;
    let mut asset_roots = Vec::new();
    let mut quality_tiers = Vec::new();
    let mut geometry_sources = Vec::new();
    let mut geometry_source_ids = Vec::new();
    let mut shading_model_ids = Vec::new();
    let mut permutation_registries = Vec::new();
    let mut resource_registry = None;
    let mut export_resource_registry = None;
    let mut cache_dir = None;
    let mut report = None;
    let mut builtin_fallback = false;
    let mut validate_wgpu_modules = false;
    let mut validate_wgpu_pipelines = false;
    let mut execution_budget = ShaderVariantPrewarmExecutionBudget::default();
    let mut pretty = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg_text = arg.to_str().ok_or_else(|| {
            ShaderPrewarmArgsError::Usage(
                "zircon_shader_prewarm expects UTF-8 command arguments".to_string(),
            )
        })?;
        match arg_text {
            "-h" | "--help" => return Ok(None),
            "--project-root" => project_root = next_path(&mut args, "--project-root")?,
            "--manifest" => manifest = Some(next_path(&mut args, "--manifest")?),
            "--asset-root" => asset_roots.push(next_path(&mut args, "--asset-root")?),
            "--quality-tier" => {
                let value = next_string(&mut args, "--quality-tier")?;
                quality_tiers.extend(parse_quality_tier(&value)?);
            }
            "--geometry-source" => {
                let value = next_string(&mut args, "--geometry-source")?;
                geometry_sources.extend(parse_geometry_source(&value)?);
            }
            "--geometry-source-id" => {
                let value = next_string(&mut args, "--geometry-source-id")?;
                geometry_source_ids.push(parse_geometry_source_id(&value)?);
            }
            "--shading-model-id" => {
                let value = next_string(&mut args, "--shading-model-id")?;
                shading_model_ids.push(parse_shading_model_id(&value)?);
            }
            "--shader-permutation-registry" => {
                permutation_registries.push(next_path(&mut args, "--shader-permutation-registry")?);
            }
            "--resource-registry" => {
                resource_registry = Some(next_path(&mut args, "--resource-registry")?);
            }
            "--export-resource-registry" => {
                export_resource_registry =
                    Some(next_path(&mut args, "--export-resource-registry")?);
            }
            "--cache-dir" => cache_dir = Some(next_path(&mut args, "--cache-dir")?),
            "--report" => report = Some(next_path(&mut args, "--report")?),
            "--builtin-fallback" => builtin_fallback = true,
            "--validate-wgpu-modules" => validate_wgpu_modules = true,
            "--validate-wgpu-pipelines" => validate_wgpu_pipelines = true,
            "--max-in-flight-variants" => {
                execution_budget.max_in_flight_variants =
                    next_usize(&mut args, "--max-in-flight-variants")?;
            }
            "--max-in-flight-source-bytes" => {
                execution_budget.max_in_flight_source_bytes =
                    next_usize(&mut args, "--max-in-flight-source-bytes")?;
            }
            "--max-resident-source-bytes" => {
                execution_budget.max_resident_source_bytes =
                    next_usize(&mut args, "--max-resident-source-bytes")?;
            }
            "--pretty" => pretty = true,
            unknown => {
                return Err(ShaderPrewarmArgsError::Usage(usage(&format!(
                    "unknown argument {unknown}"
                ))));
            }
        }
    }

    if manifest.is_none() && asset_roots.is_empty() && !builtin_fallback {
        return Err(ShaderPrewarmArgsError::Usage(usage(
            "missing --manifest, --asset-root, or --builtin-fallback",
        )));
    }
    let quality_tiers = normalized_quality_tiers(quality_tiers);
    let geometry_source_ids = normalized_geometry_source_ids(geometry_source_ids)?;
    let mut explicit_geometry_sources = geometry_source_ids.values().copied().collect::<Vec<_>>();
    explicit_geometry_sources.sort_by_key(|geometry_source| geometry_source.value());
    geometry_sources.extend(explicit_geometry_sources);
    let geometry_sources = normalized_geometry_sources(geometry_sources);
    let shading_model_ids = normalized_shading_model_ids(shading_model_ids)?;

    Ok(Some(ShaderPrewarmArgs {
        project_root,
        manifest,
        asset_roots,
        quality_tiers,
        geometry_sources,
        geometry_source_ids,
        shading_model_ids,
        permutation_registries,
        resource_registry,
        export_resource_registry,
        cache_dir,
        report,
        builtin_fallback,
        validate_wgpu_modules,
        validate_wgpu_pipelines,
        execution_budget,
        pretty,
    }))
}

pub fn usage(message: &str) -> String {
    format!(
        "{message}\nusage: zircon_shader_prewarm [--project-root <dir>] [--manifest <manifest.json>] [--asset-root <dir>]... [--quality-tier low|medium|high|ultra|all]... [--geometry-source static|skinned|morphed|skinned-morphed|all]... [--geometry-source-id <custom:name>=<4-255>]... [--shading-model-id <custom:name>=<16-255>]... [--shader-permutation-registry <registry.json>]... [--resource-registry <records.json>] [--export-resource-registry <records.json>] [--cache-dir <dir>] [--report <path>] [--builtin-fallback] [--validate-wgpu-modules] [--validate-wgpu-pipelines] [--max-in-flight-variants 1] [--max-in-flight-source-bytes <bytes>] [--max-resident-source-bytes <bytes>] [--pretty]"
    )
}

fn next_path(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> ShaderPrewarmArgsResult<PathBuf> {
    next_string(args, flag).map(PathBuf::from)
}

fn next_string(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> ShaderPrewarmArgsResult<String> {
    args.next()
        .ok_or_else(|| ShaderPrewarmArgsError::Usage(usage(&format!("missing value for {flag}"))))?
        .into_string()
        .map_err(|_| ShaderPrewarmArgsError::Usage(usage(&format!("{flag} value must be UTF-8"))))
}

fn next_usize(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> ShaderPrewarmArgsResult<usize> {
    let value = next_string(args, flag)?;
    value.parse::<usize>().map_err(|_| {
        ShaderPrewarmArgsError::Usage(usage(&format!(
            "{flag} must be a positive byte or worker count, got {value}"
        )))
    })
}

fn parse_quality_tier(value: &str) -> ShaderPrewarmArgsResult<Vec<ShaderQualityTier>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "low" => Ok(vec![ShaderQualityTier::Low]),
        "medium" => Ok(vec![ShaderQualityTier::Medium]),
        "high" => Ok(vec![ShaderQualityTier::High]),
        "ultra" => Ok(vec![ShaderQualityTier::Ultra]),
        "all" => Ok(vec![
            ShaderQualityTier::Low,
            ShaderQualityTier::Medium,
            ShaderQualityTier::High,
            ShaderQualityTier::Ultra,
        ]),
        _ => Err(ShaderPrewarmArgsError::Usage(usage(&format!(
            "unknown shader quality tier {value}; expected low, medium, high, ultra, or all"
        )))),
    }
}

fn parse_geometry_source(value: &str) -> ShaderPrewarmArgsResult<Vec<GeometrySourceId>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "static" | "static_mesh" | "static-mesh" => Ok(vec![GEOMETRY_SOURCE_ID_STATIC_MESH]),
        "skinned" | "skinned_mesh" | "skinned-mesh" => Ok(vec![GEOMETRY_SOURCE_ID_SKINNED_MESH]),
        "morphed" | "morphed_mesh" | "morphed-mesh" => Ok(vec![GEOMETRY_SOURCE_ID_MORPHED_MESH]),
        "skinned_morphed" | "skinned-morphed" | "skinned_morphed_mesh" | "skinned-morphed-mesh" => {
            Ok(vec![GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH])
        }
        "all" => Ok(builtin_geometry_source_descriptors()
            .into_iter()
            .map(|descriptor| descriptor.id)
            .collect()),
        _ => Err(ShaderPrewarmArgsError::Usage(usage(&format!(
            "unknown geometry source {value}; expected static, skinned, morphed, skinned-morphed, or all"
        )))),
    }
}

fn parse_geometry_source_id(value: &str) -> ShaderPrewarmArgsResult<(String, GeometrySourceId)> {
    let (token, id) = value.split_once('=').ok_or_else(|| {
        ShaderPrewarmArgsError::Usage(usage(&format!(
            "invalid geometry source id {value}; expected <custom:name>=<4-255>"
        )))
    })?;
    let token = normalized_custom_geometry_source_token(token)?;
    let id = id.trim().parse::<u8>().map_err(|_| {
        ShaderPrewarmArgsError::Usage(usage(&format!(
            "invalid geometry source id {value}; expected numeric plugin id 4-255"
        )))
    })?;
    if id < GEOMETRY_SOURCE_PLUGIN_ID_START {
        return Err(ShaderPrewarmArgsError::Usage(usage(&format!(
            "invalid geometry source id {value}; plugin geometry source ids must be >= {GEOMETRY_SOURCE_PLUGIN_ID_START}"
        ))));
    }
    Ok((token, GeometrySourceId::new(id)))
}

pub(crate) fn normalized_custom_geometry_source_token(
    token: &str,
) -> ShaderPrewarmArgsResult<String> {
    let token = token.trim().to_ascii_lowercase();
    if token.is_empty() {
        return Err(ShaderPrewarmArgsError::Usage(usage(
            "custom geometry source token must not be empty",
        )));
    }
    if let Some(name) = token.strip_prefix("custom:") {
        let name = name.trim();
        if name.is_empty() {
            return Err(ShaderPrewarmArgsError::Usage(usage(
                "custom geometry source token must not be empty",
            )));
        }
        return Ok(format!("custom:{name}"));
    }
    Ok(format!("custom:{token}"))
}

fn parse_shading_model_id(value: &str) -> ShaderPrewarmArgsResult<(String, ShadingModelId)> {
    let (token, id) = value.split_once('=').ok_or_else(|| {
        ShaderPrewarmArgsError::Usage(usage(&format!(
            "invalid shading model id {value}; expected <custom:name>=<16-255>"
        )))
    })?;
    let token = normalized_custom_shading_model_token(token)?;
    let id = id.trim().parse::<u8>().map_err(|_| {
        ShaderPrewarmArgsError::Usage(usage(&format!(
            "invalid shading model id {value}; expected numeric plugin id 16-255"
        )))
    })?;
    if id < SHADING_MODEL_PLUGIN_ID_START {
        return Err(ShaderPrewarmArgsError::Usage(usage(&format!(
            "invalid shading model id {value}; plugin shading model ids must be >= {SHADING_MODEL_PLUGIN_ID_START}"
        ))));
    }
    Ok((token, ShadingModelId::new(id)))
}

pub(crate) fn normalized_custom_shading_model_token(
    token: &str,
) -> ShaderPrewarmArgsResult<String> {
    let token = token.trim().to_ascii_lowercase();
    if token.is_empty() {
        return Err(ShaderPrewarmArgsError::Usage(usage(
            "custom shading model token must not be empty",
        )));
    }
    if let Some(name) = token.strip_prefix("custom:") {
        let name = name.trim();
        if name.is_empty() {
            return Err(ShaderPrewarmArgsError::Usage(usage(
                "custom shading model token must not be empty",
            )));
        }
        return Ok(format!("custom:{name}"));
    }
    Ok(format!("custom:{token}"))
}

fn normalized_quality_tiers(mut quality_tiers: Vec<ShaderQualityTier>) -> Vec<ShaderQualityTier> {
    if quality_tiers.is_empty() {
        quality_tiers.push(ShaderQualityTier::Medium);
    }
    let mut seen = HashSet::new();
    quality_tiers
        .into_iter()
        .filter(|quality| seen.insert(*quality))
        .collect()
}

fn normalized_shading_model_ids(
    shading_model_ids: Vec<(String, ShadingModelId)>,
) -> ShaderPrewarmArgsResult<BTreeMap<String, ShadingModelId>> {
    let mut by_token = BTreeMap::new();
    let mut by_id = BTreeMap::new();
    for (token, id) in shading_model_ids {
        if let Some(existing_id) = by_token.get(&token) {
            if *existing_id != id {
                return Err(ShaderPrewarmArgsError::Usage(usage(&format!(
                    "custom shading model {token} was assigned both id {existing_id} and id {id}"
                ))));
            }
            continue;
        }
        if let Some(existing_token) = by_id.get(&id) {
            return Err(ShaderPrewarmArgsError::Usage(usage(&format!(
                "custom shading model id {id} is already assigned to {existing_token} and cannot be reused by {token}"
            ))));
        }
        by_id.insert(id, token.clone());
        by_token.insert(token, id);
    }
    Ok(by_token)
}

fn normalized_geometry_source_ids(
    geometry_source_ids: Vec<(String, GeometrySourceId)>,
) -> ShaderPrewarmArgsResult<BTreeMap<String, GeometrySourceId>> {
    let mut by_token: BTreeMap<String, GeometrySourceId> = BTreeMap::new();
    let mut by_id: BTreeMap<u8, String> = BTreeMap::new();
    for (token, id) in geometry_source_ids {
        if let Some(existing_id) = by_token.get(&token) {
            if *existing_id != id {
                return Err(ShaderPrewarmArgsError::Usage(usage(&format!(
                    "custom geometry source {token} was assigned both id {} and id {}",
                    existing_id.value(),
                    id.value()
                ))));
            }
            continue;
        }
        if let Some(existing_token) = by_id.get(&id.value()) {
            return Err(ShaderPrewarmArgsError::Usage(usage(&format!(
                "custom geometry source id {} is already assigned to {existing_token} and cannot be reused by {token}",
                id.value()
            ))));
        }
        by_id.insert(id.value(), token.clone());
        by_token.insert(token, id);
    }
    Ok(by_token)
}

fn normalized_geometry_sources(
    mut geometry_sources: Vec<GeometrySourceId>,
) -> Vec<GeometrySourceId> {
    if geometry_sources.is_empty() {
        geometry_sources.push(GEOMETRY_SOURCE_ID_STATIC_MESH);
    }
    let mut seen = HashSet::new();
    geometry_sources
        .into_iter()
        .filter(|geometry_source| seen.insert(*geometry_source))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use zircon_runtime::core::framework::render::{
        GeometrySourceId, GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
        GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
        GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START,
    };

    use super::super::error::ShaderPrewarmArgsError;
    use super::parse;

    #[test]
    fn shader_prewarm_args_default_to_static_geometry_source() {
        let args = parse(["--asset-root", "assets"].into_iter().map(OsString::from))
            .unwrap()
            .unwrap();

        assert_eq!(args.geometry_sources, vec![GEOMETRY_SOURCE_ID_STATIC_MESH]);
    }

    #[test]
    fn shader_prewarm_args_expand_all_builtin_geometry_sources() {
        let args = parse(
            ["--asset-root", "assets", "--geometry-source", "all"]
                .into_iter()
                .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            args.geometry_sources,
            vec![
                GEOMETRY_SOURCE_ID_STATIC_MESH,
                GEOMETRY_SOURCE_ID_SKINNED_MESH,
                GEOMETRY_SOURCE_ID_MORPHED_MESH,
                GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
            ]
        );
    }

    #[test]
    fn shader_prewarm_args_parse_custom_shading_model_plugin_ids() {
        let args = parse(
            [
                "--asset-root",
                "assets",
                "--shading-model-id",
                "custom:Subsurface=16",
                "--shading-model-id",
                "toon=17",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            args.shading_model_ids
                .get("custom:subsurface")
                .copied()
                .unwrap()
                .value(),
            SHADING_MODEL_PLUGIN_ID_START
        );
        assert_eq!(
            args.shading_model_ids
                .get("custom:toon")
                .copied()
                .unwrap()
                .value(),
            SHADING_MODEL_PLUGIN_ID_START + 1
        );
    }

    #[test]
    fn shader_prewarm_args_parse_custom_geometry_source_plugin_ids() {
        let args = parse(
            [
                "--asset-root",
                "assets",
                "--geometry-source-id",
                "custom:GpuDriven=4",
                "--geometry-source-id",
                "foliage=5",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            args.geometry_source_ids
                .get("custom:gpudriven")
                .copied()
                .unwrap()
                .value(),
            GEOMETRY_SOURCE_PLUGIN_ID_START
        );
        assert_eq!(
            args.geometry_source_ids
                .get("custom:foliage")
                .copied()
                .unwrap()
                .value(),
            GEOMETRY_SOURCE_PLUGIN_ID_START + 1
        );
        assert_eq!(
            args.geometry_sources,
            vec![
                GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START),
                GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START + 1),
            ]
        );
    }

    #[test]
    fn shader_prewarm_args_parse_resource_registry_path() {
        let args = parse(
            [
                "--asset-root",
                "assets",
                "--resource-registry",
                "Project/.zircon/cache/resources.json",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            args.resource_registry.unwrap(),
            std::path::PathBuf::from("Project/.zircon/cache/resources.json")
        );
    }

    #[test]
    fn shader_prewarm_args_parse_shader_permutation_registry_path() {
        let args = parse(
            [
                "--asset-root",
                "assets",
                "--shader-permutation-registry",
                "Project/.zircon/cache/shader_permutation_registry.json",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            args.permutation_registries,
            vec![std::path::PathBuf::from(
                "Project/.zircon/cache/shader_permutation_registry.json"
            )]
        );
    }

    #[test]
    fn shader_prewarm_args_parse_export_resource_registry_path() {
        let args = parse(
            [
                "--asset-root",
                "assets",
                "--export-resource-registry",
                "cache/shader_resource_records.json",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            args.export_resource_registry.unwrap(),
            std::path::PathBuf::from("cache/shader_resource_records.json")
        );
    }

    #[test]
    fn shader_prewarm_args_parse_wgpu_module_validation_flag() {
        let args = parse(
            ["--asset-root", "assets", "--validate-wgpu-modules"]
                .into_iter()
                .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert!(args.validate_wgpu_modules);
    }

    #[test]
    fn shader_prewarm_args_parse_wgpu_pipeline_validation_flag() {
        let args = parse(
            ["--asset-root", "assets", "--validate-wgpu-pipelines"]
                .into_iter()
                .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert!(args.validate_wgpu_pipelines);
    }

    #[test]
    fn shader_prewarm_args_reject_builtin_shading_model_id_range() {
        let error = parse(
            [
                "--asset-root",
                "assets",
                "--shading-model-id",
                "custom:subsurface=2",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap_err();

        assert!(matches!(error, ShaderPrewarmArgsError::Usage(_)));
        assert!(error
            .to_string()
            .contains("plugin shading model ids must be >= 16"));
    }

    #[test]
    fn shader_prewarm_args_reject_builtin_geometry_source_id_range() {
        let error = parse(
            [
                "--asset-root",
                "assets",
                "--geometry-source-id",
                "custom:gpu-driven=3",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap_err();

        assert!(matches!(error, ShaderPrewarmArgsError::Usage(_)));
        assert!(error
            .to_string()
            .contains("plugin geometry source ids must be >= 4"));
    }

    #[test]
    fn shader_prewarm_args_missing_value_reports_typed_usage_error() {
        let error = parse(["--asset-root"].into_iter().map(OsString::from)).unwrap_err();

        assert!(matches!(error, ShaderPrewarmArgsError::Usage(_)));
        assert!(error.to_string().contains("missing value for --asset-root"));
    }
}

use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;

use zircon_runtime::core::framework::render::{
    builtin_geometry_source_descriptors, GeometrySourceId, ShaderQualityTier, ShadingModelId,
    GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADING_MODEL_PLUGIN_ID_START,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderPrewarmArgs {
    pub project_root: PathBuf,
    pub manifest: Option<PathBuf>,
    pub asset_roots: Vec<PathBuf>,
    pub quality_tiers: Vec<ShaderQualityTier>,
    pub geometry_sources: Vec<GeometrySourceId>,
    pub shading_model_ids: BTreeMap<String, ShadingModelId>,
    pub resource_registry: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub report: Option<PathBuf>,
    pub builtin_fallback: bool,
    pub pretty: bool,
}

pub fn parse(
    args: impl IntoIterator<Item = OsString>,
) -> Result<Option<ShaderPrewarmArgs>, String> {
    let mut project_root = PathBuf::from(".");
    let mut manifest = None;
    let mut asset_roots = Vec::new();
    let mut quality_tiers = Vec::new();
    let mut geometry_sources = Vec::new();
    let mut shading_model_ids = Vec::new();
    let mut resource_registry = None;
    let mut cache_dir = None;
    let mut report = None;
    let mut builtin_fallback = false;
    let mut pretty = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg_text = arg
            .to_str()
            .ok_or_else(|| "zircon_shader_prewarm expects UTF-8 command arguments".to_string())?;
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
            "--shading-model-id" => {
                let value = next_string(&mut args, "--shading-model-id")?;
                shading_model_ids.push(parse_shading_model_id(&value)?);
            }
            "--resource-registry" => {
                resource_registry = Some(next_path(&mut args, "--resource-registry")?);
            }
            "--cache-dir" => cache_dir = Some(next_path(&mut args, "--cache-dir")?),
            "--report" => report = Some(next_path(&mut args, "--report")?),
            "--builtin-fallback" => builtin_fallback = true,
            "--pretty" => pretty = true,
            unknown => return Err(usage(&format!("unknown argument {unknown}"))),
        }
    }

    if manifest.is_none() && asset_roots.is_empty() && !builtin_fallback {
        return Err(usage(
            "missing --manifest, --asset-root, or --builtin-fallback",
        ));
    }
    let quality_tiers = normalized_quality_tiers(quality_tiers);
    let geometry_sources = normalized_geometry_sources(geometry_sources);
    let shading_model_ids = normalized_shading_model_ids(shading_model_ids)?;

    Ok(Some(ShaderPrewarmArgs {
        project_root,
        manifest,
        asset_roots,
        quality_tiers,
        geometry_sources,
        shading_model_ids,
        resource_registry,
        cache_dir,
        report,
        builtin_fallback,
        pretty,
    }))
}

pub fn usage(message: &str) -> String {
    format!(
        "{message}\nusage: zircon_shader_prewarm [--project-root <dir>] [--manifest <manifest.json>] [--asset-root <dir>]... [--quality-tier low|medium|high|ultra|all]... [--geometry-source static|skinned|morphed|skinned-morphed|all]... [--shading-model-id <custom:name>=<16-255>]... [--resource-registry <records.json>] [--cache-dir <dir>] [--report <path>] [--builtin-fallback] [--pretty]"
    )
}

fn next_path(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> Result<PathBuf, String> {
    next_string(args, flag).map(PathBuf::from)
}

fn next_string(
    args: &mut impl Iterator<Item = OsString>,
    flag: &'static str,
) -> Result<String, String> {
    args.next()
        .ok_or_else(|| usage(&format!("missing value for {flag}")))?
        .into_string()
        .map_err(|_| usage(&format!("{flag} value must be UTF-8")))
}

fn parse_quality_tier(value: &str) -> Result<Vec<ShaderQualityTier>, String> {
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
        _ => Err(usage(&format!(
            "unknown shader quality tier {value}; expected low, medium, high, ultra, or all"
        ))),
    }
}

fn parse_geometry_source(value: &str) -> Result<Vec<GeometrySourceId>, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "static" | "static_mesh" | "static-mesh" => Ok(vec![GEOMETRY_SOURCE_ID_STATIC_MESH]),
        "skinned" | "skinned_mesh" | "skinned-mesh" => Ok(vec![GEOMETRY_SOURCE_ID_SKINNED_MESH]),
        "morphed" | "morphed_mesh" | "morphed-mesh" => Ok(vec![GEOMETRY_SOURCE_ID_MORPHED_MESH]),
        "skinned_morphed" | "skinned-morphed" | "skinned_morphed_mesh"
        | "skinned-morphed-mesh" => Ok(vec![GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH]),
        "all" => Ok(builtin_geometry_source_descriptors()
            .into_iter()
            .map(|descriptor| descriptor.id)
            .collect()),
        _ => Err(usage(&format!(
            "unknown geometry source {value}; expected static, skinned, morphed, skinned-morphed, or all"
        ))),
    }
}

fn parse_shading_model_id(value: &str) -> Result<(String, ShadingModelId), String> {
    let (token, id) = value.split_once('=').ok_or_else(|| {
        usage(&format!(
            "invalid shading model id {value}; expected <custom:name>=<16-255>"
        ))
    })?;
    let token = normalized_custom_shading_model_token(token)?;
    let id = id.trim().parse::<u8>().map_err(|_| {
        usage(&format!(
            "invalid shading model id {value}; expected numeric plugin id 16-255"
        ))
    })?;
    if id < SHADING_MODEL_PLUGIN_ID_START {
        return Err(usage(&format!(
            "invalid shading model id {value}; plugin shading model ids must be >= {SHADING_MODEL_PLUGIN_ID_START}"
        )));
    }
    Ok((token, ShadingModelId::new(id)))
}

fn normalized_custom_shading_model_token(token: &str) -> Result<String, String> {
    let token = token.trim().to_ascii_lowercase();
    if token.is_empty() {
        return Err(usage("custom shading model token must not be empty"));
    }
    if let Some(name) = token.strip_prefix("custom:") {
        let name = name.trim();
        if name.is_empty() {
            return Err(usage("custom shading model token must not be empty"));
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
) -> Result<BTreeMap<String, ShadingModelId>, String> {
    let mut by_token = BTreeMap::new();
    let mut by_id = BTreeMap::new();
    for (token, id) in shading_model_ids {
        if let Some(existing_id) = by_token.get(&token) {
            if *existing_id != id {
                return Err(usage(&format!(
                    "custom shading model {token} was assigned both id {existing_id} and id {id}"
                )));
            }
            continue;
        }
        if let Some(existing_token) = by_id.get(&id) {
            return Err(usage(&format!(
                "custom shading model id {id} is already assigned to {existing_token} and cannot be reused by {token}"
            )));
        }
        by_id.insert(id, token.clone());
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
        GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
        GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
        SHADING_MODEL_PLUGIN_ID_START,
    };

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
    fn shader_prewarm_args_parse_resource_registry_path() {
        let args = parse(
            [
                "--asset-root",
                "assets",
                "--resource-registry",
                "Project/library/resources.json",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            args.resource_registry.unwrap(),
            std::path::PathBuf::from("Project/library/resources.json")
        );
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

        assert!(error.contains("plugin shading model ids must be >= 16"));
    }
}

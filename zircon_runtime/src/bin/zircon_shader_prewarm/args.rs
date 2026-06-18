use std::collections::HashSet;
use std::ffi::OsString;
use std::path::PathBuf;

use zircon_runtime::core::framework::render::ShaderQualityTier;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderPrewarmArgs {
    pub project_root: PathBuf,
    pub manifest: Option<PathBuf>,
    pub asset_roots: Vec<PathBuf>,
    pub quality_tiers: Vec<ShaderQualityTier>,
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

    Ok(Some(ShaderPrewarmArgs {
        project_root,
        manifest,
        asset_roots,
        quality_tiers,
        cache_dir,
        report,
        builtin_fallback,
        pretty,
    }))
}

pub fn usage(message: &str) -> String {
    format!(
        "{message}\nusage: zircon_shader_prewarm [--project-root <dir>] [--manifest <manifest.json>] [--asset-root <dir>]... [--quality-tier low|medium|high|ultra|all]... [--cache-dir <dir>] [--report <path>] [--builtin-fallback] [--pretty]"
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

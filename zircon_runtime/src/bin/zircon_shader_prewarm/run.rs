use std::ffi::OsString;
use std::fs;
use std::process::ExitCode;

use zircon_runtime::core::framework::render::ShaderVariantPrewarmManifest;
use zircon_runtime::dynamic_api::{
    default_shader_variant_cache_root_for_project, prewarm_shader_variants,
};

use super::args::{parse, usage};
use super::manifest::{
    asset_root_manifest_with_resource_registry_revisions,
    builtin_fallback_manifest_for_quality_tiers_and_geometry_sources, merge_manifests,
    read_manifest, resource_registry::ShaderPrewarmResourceRegistryOverlay,
};

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<ExitCode, String> {
    let Some(args) = parse(args)? else {
        println!("{}", usage("zircon shader variant prewarm tool"));
        return Ok(ExitCode::SUCCESS);
    };

    let mut manifest = ShaderVariantPrewarmManifest::new(Vec::new());
    if args.builtin_fallback {
        manifest = merge_manifests(
            manifest,
            builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(
                &args.quality_tiers,
                &args.geometry_sources,
            ),
        )?;
    }
    if let Some(path) = &args.manifest {
        manifest = merge_manifests(manifest, read_manifest(path)?)?;
    }
    let resource_registry = args
        .resource_registry
        .as_deref()
        .map(ShaderPrewarmResourceRegistryOverlay::read)
        .transpose()?;
    for asset_root in &args.asset_roots {
        manifest = merge_manifests(
            manifest,
            asset_root_manifest_with_resource_registry_revisions(
                asset_root,
                &args.quality_tiers,
                &args.geometry_sources,
                &args.shading_model_ids,
                resource_registry.as_ref(),
            )?,
        )?;
    }

    let cache_dir = args
        .cache_dir
        .unwrap_or_else(|| default_shader_variant_cache_root_for_project(&args.project_root));
    let report = prewarm_shader_variants(&manifest, &cache_dir);
    let json = if args.pretty {
        serde_json::to_string_pretty(&report)
    } else {
        serde_json::to_string(&report)
    }
    .map_err(|error| format!("failed to encode shader prewarm report: {error}"))?;

    if let Some(report_path) = &args.report {
        if let Some(parent) = report_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create shader prewarm report directory {}: {error}",
                        parent.display()
                    )
                })?;
            }
        }
        fs::write(report_path, &json).map_err(|error| {
            format!(
                "failed to write shader prewarm report {}: {error}",
                report_path.display()
            )
        })?;
    }

    println!("{json}");
    if report.failed_count > 0 {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

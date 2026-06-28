use std::ffi::OsString;
use std::fs;
use std::process::ExitCode;

use zircon_runtime::core::framework::render::ShaderVariantPrewarmManifest;
use zircon_runtime::dynamic_api::{
    default_shader_variant_cache_root_for_project, prewarm_shader_variants,
    prewarm_shader_variants_with_wgpu_module_validation,
};

use super::args::{parse, usage};
use super::manifest::{
    asset_root_manifest_with_resource_registry_revisions,
    builtin_fallback_manifest_for_quality_tiers_and_geometry_sources, merge_manifests,
    permutation_registry::{
        shader_permutation_registry_paths, ShaderPrewarmPermutationRegistryOverlay,
    },
    read_manifest,
    resource_registry::{
        shader_resource_records_from_asset_roots, ShaderPrewarmResourceRegistryOverlay,
    },
};

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<ExitCode, String> {
    let Some(args) = parse(args)? else {
        println!("{}", usage("zircon shader variant prewarm tool"));
        return Ok(ExitCode::SUCCESS);
    };

    let mut geometry_sources = args.geometry_sources.clone();
    let mut geometry_source_ids = args.geometry_source_ids.clone();
    let mut shading_model_ids = args.shading_model_ids.clone();
    for registry_path in
        shader_permutation_registry_paths(&args.permutation_registries, &args.asset_roots)
    {
        ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)?.merge_into(
            &mut geometry_sources,
            &mut geometry_source_ids,
            &mut shading_model_ids,
        )?;
    }

    let mut manifest = ShaderVariantPrewarmManifest::new(Vec::new());
    if args.builtin_fallback {
        manifest = merge_manifests(
            manifest,
            builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(
                &args.quality_tiers,
                &geometry_sources,
            ),
        )?;
    }
    if let Some(path) = &args.manifest {
        manifest = merge_manifests(manifest, read_manifest(path)?)?;
    }
    let exported_resource_records = export_shader_resource_registry_for_asset_roots(
        &args.asset_roots,
        args.export_resource_registry.as_ref(),
    )?;
    let resource_registry = if let Some(path) = args.resource_registry.as_deref() {
        Some(ShaderPrewarmResourceRegistryOverlay::read(path)?)
    } else {
        exported_resource_records
            .clone()
            .map(ShaderPrewarmResourceRegistryOverlay::from_records)
    };
    for asset_root in &args.asset_roots {
        manifest = merge_manifests(
            manifest,
            asset_root_manifest_with_resource_registry_revisions(
                asset_root,
                &args.quality_tiers,
                &geometry_sources,
                &shading_model_ids,
                resource_registry.as_ref(),
            )?,
        )?;
    }

    let cache_dir = args
        .cache_dir
        .unwrap_or_else(|| default_shader_variant_cache_root_for_project(&args.project_root));
    let report = if args.validate_wgpu_modules {
        prewarm_shader_variants_with_wgpu_module_validation(&manifest, &cache_dir)
    } else {
        prewarm_shader_variants(&manifest, &cache_dir)
    };
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

fn export_shader_resource_registry_for_asset_roots(
    asset_roots: &[std::path::PathBuf],
    export_path: Option<&std::path::PathBuf>,
) -> Result<Option<Vec<zircon_runtime::core::resource::ResourceRecord>>, String> {
    let Some(export_path) = export_path else {
        return Ok(None);
    };
    let records = shader_resource_records_from_asset_roots(asset_roots)?;
    let json = serde_json::json!({ "resources": records });
    let json = serde_json::to_string_pretty(&json)
        .map_err(|error| format!("failed to encode shader resource registry: {error}"))?;
    if let Some(parent) = export_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create shader resource registry directory {}: {error}",
                    parent.display()
                )
            })?;
        }
    }
    fs::write(export_path, json).map_err(|error| {
        format!(
            "failed to write shader resource registry {}: {error}",
            export_path.display()
        )
    })?;
    Ok(Some(records))
}

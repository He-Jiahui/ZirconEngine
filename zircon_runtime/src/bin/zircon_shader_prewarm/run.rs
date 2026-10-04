use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use zircon_runtime::core::framework::render::{
    ShaderQualityTier, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use zircon_runtime::dynamic_api::{
    default_shader_variant_cache_root_for_project, prewarm_shader_variants_with_execution_budget,
};

use super::args::{parse, usage};
use super::error::{
    ShaderPrewarmReportError, ShaderPrewarmReportResult, ShaderPrewarmResourceRegistryError,
    ShaderPrewarmResourceRegistryResult,
};
use super::manifest::{
    asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs,
    builtin_fallback_manifest_for_quality_tiers_and_geometry_sources, merge_manifests,
    permutation_registry::{
        shader_permutation_registry_paths, ShaderPrewarmPermutationRegistryOverlay,
    },
    read_manifest,
    resource_registry::{
        shader_resource_records_from_asset_roots,
        shader_resource_records_from_loaded_meta_document_refs,
        ShaderPrewarmResourceRegistryOverlay,
    },
    ShaderPrewarmAssetInventory,
};

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<ExitCode, String> {
    let Some(args) = parse(args).map_err(|error| error.to_string())? else {
        println!("{}", usage("zircon shader variant prewarm tool"));
        return Ok(ExitCode::SUCCESS);
    };
    if args.execution_budget.validate().is_err() {
        let report = prewarm_shader_variants_with_execution_budget(
            &ShaderVariantPrewarmManifest::empty(),
            &args.project_root,
            args.execution_budget,
            false,
            false,
        );
        return finish_shader_prewarm_report(&report, args.report.as_deref(), args.pretty);
    }

    let mut geometry_sources = args.geometry_sources.clone();
    let mut geometry_source_ids = args.geometry_source_ids.clone();
    let mut geometry_source_descriptors = BTreeMap::new();
    let mut shading_model_ids = args.shading_model_ids.clone();
    let mut shading_model_descriptors = BTreeMap::new();
    let mut shader_modules = BTreeMap::new();
    let permutation_registry_paths =
        shader_permutation_registry_paths(&args.permutation_registries, &args.asset_roots);
    let has_permutation_registry = !permutation_registry_paths.is_empty();
    for registry_path in permutation_registry_paths {
        let registry_overlay = ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
            .map_err(|error| error.to_string())?;
        registry_overlay
            .merge_into(
                &mut geometry_sources,
                &mut geometry_source_ids,
                &mut geometry_source_descriptors,
                &mut shading_model_ids,
                &mut shading_model_descriptors,
                &mut shader_modules,
            )
            .map_err(|error| error.to_string())?;
    }
    let has_external_permutation_inputs = has_permutation_registry
        || args.quality_tiers != [ShaderQualityTier::Medium]
        || geometry_sources != [GEOMETRY_SOURCE_ID_STATIC_MESH]
        || !geometry_source_ids.is_empty()
        || !shading_model_ids.is_empty()
        || !geometry_source_descriptors.is_empty()
        || !shading_model_descriptors.is_empty()
        || !shader_modules.is_empty();

    let mut manifest = ShaderVariantPrewarmManifest::empty();
    if args.builtin_fallback {
        manifest = merge_manifests(
            manifest,
            builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(
                &args.quality_tiers,
                &geometry_sources,
                &geometry_source_descriptors,
            ),
        )
        .map_err(|error| error.to_string())?;
    }
    if let Some(path) = &args.manifest {
        let manifest_from_file = read_manifest(path).map_err(|error| error.to_string())?;
        manifest =
            merge_manifests(manifest, manifest_from_file).map_err(|error| error.to_string())?;
    }
    let cache_dir = args
        .cache_dir
        .unwrap_or_else(|| default_shader_variant_cache_root_for_project(&args.project_root));
    fs::create_dir_all(&cache_dir).map_err(|error| {
        format!(
            "failed to create shader variant cache directory `{}`: {error}",
            cache_dir.display()
        )
    })?;
    if args
        .asset_roots
        .iter()
        .any(|asset_root| cache_root_matches_asset_root(&cache_dir, asset_root))
    {
        return Err(format!(
            "shader variant cache directory `{}` must not equal an asset root; choose a separate or nested cache directory",
            cache_dir.display()
        ));
    }
    let inventory_snapshot_root = cache_dir.join("asset_inventories");
    let has_external_resource_registry = args.resource_registry.is_some();
    let needs_unchanged_inventory_payload = has_external_permutation_inputs
        || has_external_resource_registry
        || args.export_resource_registry.is_some();
    let mut asset_inventories = Vec::new();
    for asset_root in &args.asset_roots {
        if !needs_unchanged_inventory_payload
            && ShaderPrewarmAssetInventory::warm_snapshot_is_current_excluding(
                asset_root,
                &inventory_snapshot_root,
                Some(&cache_dir),
                args.execution_budget.max_resident_source_bytes,
            )
        {
            continue;
        }
        let inventory = ShaderPrewarmAssetInventory::collect_with_warm_snapshot_excluding(
            asset_root,
            &inventory_snapshot_root,
            Some(&cache_dir),
            args.execution_budget.max_resident_source_bytes,
        )
        .map_err(|error| error.to_string())?;
        asset_inventories.push((asset_root.clone(), inventory));
    }
    let exported_resource_records = export_shader_resource_registry_for_asset_inventories(
        &asset_inventories,
        args.export_resource_registry.as_ref(),
    )
    .map_err(|error| error.to_string())?;
    let resource_registry = if let Some(path) = args.resource_registry.as_deref() {
        Some(ShaderPrewarmResourceRegistryOverlay::read(path).map_err(|error| error.to_string())?)
    } else {
        exported_resource_records.map(ShaderPrewarmResourceRegistryOverlay::from_records)
    };
    for (asset_root, inventory) in &asset_inventories {
        if !asset_root_requires_prewarm_projection(
            !inventory.changed_paths().is_empty(),
            has_external_permutation_inputs,
            has_external_resource_registry,
        ) {
            continue;
        }
        manifest = merge_manifests(
            manifest,
            asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs(
                asset_root,
                inventory,
                &args.quality_tiers,
                &geometry_sources,
                &geometry_source_descriptors,
                &shading_model_ids,
                &shader_modules,
                resource_registry.as_ref(),
                has_external_permutation_inputs,
            )
            .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }

    let report = prewarm_shader_variants_with_execution_budget(
        &manifest,
        &cache_dir,
        args.execution_budget,
        args.validate_wgpu_modules,
        args.validate_wgpu_pipelines,
    );
    finish_shader_prewarm_report(&report, args.report.as_deref(), args.pretty)
}

fn finish_shader_prewarm_report(
    report: &ShaderVariantPrewarmReport,
    report_path: Option<&Path>,
    pretty: bool,
) -> Result<ExitCode, String> {
    let json = encode_shader_prewarm_report(report, pretty).map_err(|error| error.to_string())?;

    if let Some(report_path) = report_path {
        write_shader_prewarm_report(report_path, &json).map_err(|error| error.to_string())?;
    }

    println!("{json}");
    if report.failed_count > 0 || report.preflight_error.is_some() {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Local inventory changes are the only input owned by a warm asset snapshot.
/// Any module or resource overlay supplied outside that snapshot must force a
/// conservative projection because its revision can change independently.
fn asset_root_requires_prewarm_projection(
    has_changed_inventory_paths: bool,
    has_external_permutation_inputs: bool,
    has_external_resource_registry: bool,
) -> bool {
    has_changed_inventory_paths || has_external_permutation_inputs || has_external_resource_registry
}

fn cache_root_matches_asset_root(cache_root: &Path, asset_root: &Path) -> bool {
    let Ok(cache_root) = fs::canonicalize(cache_root) else {
        return false;
    };
    let Ok(asset_root) = fs::canonicalize(asset_root) else {
        return false;
    };
    cache_root == asset_root
}

fn encode_shader_prewarm_report(
    report: &ShaderVariantPrewarmReport,
    pretty: bool,
) -> ShaderPrewarmReportResult<String> {
    let result = if pretty {
        serde_json::to_string_pretty(report)
    } else {
        serde_json::to_string(report)
    };
    result.map_err(|source| ShaderPrewarmReportError::ReportEncode { source })
}

fn write_shader_prewarm_report(report_path: &Path, json: &str) -> ShaderPrewarmReportResult<()> {
    if let Some(parent) = report_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| {
                ShaderPrewarmReportError::CreateReportDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
    }
    fs::write(report_path, json).map_err(|source| ShaderPrewarmReportError::WriteReport {
        path: report_path.to_path_buf(),
        source,
    })
}

fn export_shader_resource_registry_for_asset_roots(
    asset_roots: &[PathBuf],
    export_path: Option<&PathBuf>,
) -> ShaderPrewarmResourceRegistryResult<Option<Vec<zircon_runtime::core::resource::ResourceRecord>>>
{
    let Some(export_path) = export_path else {
        return Ok(None);
    };
    let records = shader_resource_records_from_asset_roots(asset_roots)?;
    write_shader_resource_registry_export(export_path, &records)?;
    Ok(Some(records))
}

fn export_shader_resource_registry_for_asset_inventories(
    asset_inventories: &[(PathBuf, ShaderPrewarmAssetInventory)],
    export_path: Option<&PathBuf>,
) -> ShaderPrewarmResourceRegistryResult<Option<Vec<zircon_runtime::core::resource::ResourceRecord>>>
{
    let Some(export_path) = export_path else {
        return Ok(None);
    };
    let records = shader_resource_records_from_loaded_meta_document_refs(
        asset_inventories
            .iter()
            .flat_map(|(_, inventory)| inventory.metadata_by_path().values()),
    )?;
    write_shader_resource_registry_export(export_path, &records)?;
    Ok(Some(records))
}

fn write_shader_resource_registry_export(
    export_path: &Path,
    records: &[zircon_runtime::core::resource::ResourceRecord],
) -> ShaderPrewarmResourceRegistryResult<()> {
    let json = serde_json::json!({ "resources": records });
    let json = serde_json::to_string_pretty(&json)
        .map_err(|source| ShaderPrewarmResourceRegistryError::EncodeExport { source })?;
    if let Some(parent) = export_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| {
                ShaderPrewarmResourceRegistryError::CreateExportDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
    }
    fs::write(export_path, json).map_err(|source| ShaderPrewarmResourceRegistryError::WriteExport {
        path: export_path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
#[path = "tests/run.rs"]
mod tests;

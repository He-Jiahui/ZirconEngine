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
mod tests {
    use super::*;
    use zircon_runtime::core::resource::{ResourceKind, ResourceRecord, ResourceState};

    #[test]
    fn exported_resource_records_move_into_the_overlay() {
        let source = include_str!("run.rs").split_once("#[cfg(test)]").unwrap().0;
        let projection = source
            .split("let resource_registry =")
            .nth(1)
            .unwrap()
            .split("let cache_dir")
            .next()
            .unwrap();

        assert!(projection.contains("exported_resource_records.map("));
        assert!(!projection.contains("exported_resource_records\n            .clone()"));
    }

    #[test]
    fn warm_unchanged_asset_roots_skip_projection_without_external_inputs() {
        assert!(!asset_root_requires_prewarm_projection(false, false, false));
        assert!(asset_root_requires_prewarm_projection(true, false, false));
        assert!(asset_root_requires_prewarm_projection(false, true, false));
        assert!(asset_root_requires_prewarm_projection(false, false, true));
    }

    #[test]
    fn default_warm_asset_roots_skip_inventory_payload_hydration() {
        let source = include_str!("run.rs").split_once("#[cfg(test)]").unwrap().0;
        let inventory_loop = source
            .split("let inventory_snapshot_root")
            .nth(1)
            .expect("run should prepare an inventory snapshot root")
            .split("let exported_resource_records")
            .next()
            .expect("inventory collection should end before registry export");

        assert!(inventory_loop.contains("needs_unchanged_inventory_payload"));
        assert!(inventory_loop.contains("warm_snapshot_is_current_excluding"));
        assert!(inventory_loop.contains("continue;"));
        assert!(
            inventory_loop.find("warm_snapshot_is_current_excluding")
                < inventory_loop.find("collect_with_warm_snapshot_excluding"),
            "the compact index check must precede full inventory hydration"
        );
    }

    #[test]
    fn invalid_execution_budget_emits_a_json_preflight_report_before_asset_io() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_prewarm_invalid_budget_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock must be after the Unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("fixture root should be created");
        let project_root = root.join("missing_project");
        let asset_root = root.join("missing_assets");
        let report_path = root.join("preflight_report.json");

        let exit_code = run([
            "--project-root",
            project_root.to_str().expect("fixture path should be UTF-8"),
            "--asset-root",
            asset_root.to_str().expect("fixture path should be UTF-8"),
            "--report",
            report_path.to_str().expect("fixture path should be UTF-8"),
            "--max-resident-source-bytes",
            "0",
        ]
        .into_iter()
        .map(OsString::from))
        .expect("invalid budget should still produce a report");
        assert_eq!(exit_code, ExitCode::from(2));
        assert!(
            !project_root.exists() && !asset_root.exists(),
            "budget preflight must run before cache creation or asset inventory IO"
        );

        let report: serde_json::Value = serde_json::from_slice(
            &fs::read(&report_path).expect("preflight report should be written"),
        )
        .expect("preflight report should be JSON");
        assert_eq!(report["requested_count"], 0);
        assert_eq!(report["written_count"], 0);
        assert_eq!(report["failed_count"], 0);
        assert_eq!(report["failures"], serde_json::json!([]));
        assert!(report["preflight_error"]
            .as_str()
            .is_some_and(|error| error.contains("max_resident_source_bytes must be non-zero")));

        fs::remove_dir_all(root).expect("fixture root should be removed");
    }

    #[test]
    fn shader_prewarm_rejects_a_cache_root_equal_to_an_asset_root() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_prewarm_cache_root_conflict_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock must be after the Unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("fixture root should be created");
        let nested_cache_root = root.join("cache");
        fs::create_dir_all(&nested_cache_root).expect("nested cache root should be created");

        assert!(cache_root_matches_asset_root(&root, &root));
        assert!(
            !cache_root_matches_asset_root(&nested_cache_root, &root),
            "a nested cache root remains a valid asset-inventory exclusion"
        );

        fs::remove_dir_all(root).expect("fixture root should be removed");
    }

    #[test]
    fn shader_prewarm_report_write_reports_typed_directory_error() {
        let report_parent = std::env::temp_dir().join(format!(
            "zircon_shader_prewarm_report_parent_{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&report_parent);
        let _ = fs::remove_dir_all(&report_parent);
        fs::write(&report_parent, "not a directory").unwrap();
        let report_path = report_parent.join("report.json");

        let error = write_shader_prewarm_report(&report_path, "{}").unwrap_err();

        match error {
            ShaderPrewarmReportError::CreateReportDirectory { path, source: _ } => {
                assert_eq!(path, report_parent);
            }
            other => panic!("expected typed report directory error, got {other:?}"),
        }

        let _ = fs::remove_file(report_parent);
    }

    #[test]
    fn shader_prewarm_project_and_plugin_asset_roots_export_wrapped_resource_registry_file() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_prewarm_project_plugin_export_file_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let project_root = root.join("project_assets");
        let plugin_root = root.join("plugin_assets");
        let export_path = root
            .join("ZirconEngine")
            .join("cache")
            .join("shader_resource_records.json");
        write_named_shader_with_meta(
            &project_root,
            "project",
            "00000000-0000-0000-0000-000000000056",
            "res://project/shaders/project",
            "source-hash-project-export-file",
        );
        write_named_shader_with_meta(
            &plugin_root,
            "plugin",
            "00000000-0000-0000-0000-000000000057",
            "package://virtual_geometry/shaders/plugin",
            "source-hash-plugin-export-file",
        );

        let returned_records = export_shader_resource_registry_for_asset_roots(
            &[project_root, plugin_root],
            Some(&export_path),
        )
        .unwrap()
        .unwrap();
        let exported_value: serde_json::Value =
            serde_json::from_slice(&fs::read(&export_path).unwrap()).unwrap();
        let exported_records =
            serde_json::from_value::<Vec<ResourceRecord>>(exported_value["resources"].clone())
                .unwrap();

        assert_eq!(returned_records.len(), exported_records.len());
        for (returned, exported) in returned_records.iter().zip(&exported_records) {
            assert_eq!(returned.id, exported.id);
            assert_eq!(returned.primary_locator, exported.primary_locator);
            assert_eq!(returned.revision, exported.revision);
        }
        assert_eq!(exported_records.len(), 2);
        assert_eq!(
            exported_records
                .iter()
                .map(|record| record.primary_locator.to_string())
                .collect::<Vec<_>>(),
            vec![
                "package://virtual_geometry/shaders/plugin".to_string(),
                "res://project/shaders/project".to_string()
            ]
        );
        assert!(exported_records.iter().all(|record| {
            record.kind == ResourceKind::Shader
                && record.state == ResourceState::Ready
                && record.revision != 0
        }));
        let overlay = ShaderPrewarmResourceRegistryOverlay::from_records(exported_records.clone());
        for record in &exported_records {
            let label = record.primary_locator.to_string();
            assert_eq!(
                overlay.revision_for(record.id, &label),
                Some(record.revision)
            );
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn shader_prewarm_resource_registry_export_reports_typed_directory_error() {
        let export_parent = std::env::temp_dir().join(format!(
            "zircon_shader_prewarm_resource_registry_parent_{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&export_parent);
        let _ = fs::remove_dir_all(&export_parent);
        fs::write(&export_parent, "not a directory").unwrap();
        let export_path = export_parent.join("shader_resource_records.json");

        let error =
            export_shader_resource_registry_for_asset_roots(&[], Some(&export_path)).unwrap_err();

        match error {
            ShaderPrewarmResourceRegistryError::CreateExportDirectory { path, source: _ } => {
                assert_eq!(path, export_parent);
            }
            other => panic!("expected typed resource registry directory error, got {other:?}"),
        }

        let _ = fs::remove_file(export_parent);
    }

    fn write_named_shader_with_meta(
        asset_root: &Path,
        name: &str,
        id: &str,
        locator: &str,
        source_hash: &str,
    ) {
        fs::create_dir_all(asset_root.join("shaders")).unwrap();
        fs::write(
            asset_root.join("shaders").join(format!("{name}.wgsl")),
            format!("fn {name}() {{}}\n"),
        )
        .unwrap();
        fs::write(
            asset_root
                .join("shaders")
                .join(format!("{name}.wgsl.zmeta")),
            format!(
                r#"format_version = 7
uuid = "{id}"
url = "{locator}"
asset_kind = "Shader"
unit = "single"
source_digest = "{source_hash}"
"#
            ),
        )
        .unwrap();
    }
}

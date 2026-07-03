use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use zircon_runtime::core::framework::render::{
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
};
use zircon_runtime::dynamic_api::{
    default_shader_variant_cache_root_for_project, prewarm_shader_variants,
    prewarm_shader_variants_with_wgpu_module_validation,
    prewarm_shader_variants_with_wgpu_pipeline_validation,
};

use super::args::{parse, usage};
use super::error::{
    ShaderPrewarmReportError, ShaderPrewarmReportResult, ShaderPrewarmResourceRegistryError,
    ShaderPrewarmResourceRegistryResult,
};
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
    let Some(args) = parse(args).map_err(|error| error.to_string())? else {
        println!("{}", usage("zircon shader variant prewarm tool"));
        return Ok(ExitCode::SUCCESS);
    };

    let mut geometry_sources = args.geometry_sources.clone();
    let mut geometry_source_ids = args.geometry_source_ids.clone();
    let mut geometry_source_descriptors = BTreeMap::new();
    let mut shading_model_ids = args.shading_model_ids.clone();
    let mut shading_model_descriptors = BTreeMap::new();
    for registry_path in
        shader_permutation_registry_paths(&args.permutation_registries, &args.asset_roots)
    {
        let registry_overlay = ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
            .map_err(|error| error.to_string())?;
        registry_overlay
            .merge_into(
                &mut geometry_sources,
                &mut geometry_source_ids,
                &mut geometry_source_descriptors,
                &mut shading_model_ids,
                &mut shading_model_descriptors,
            )
            .map_err(|error| error.to_string())?;
    }

    let mut manifest = ShaderVariantPrewarmManifest::new(Vec::new());
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
    let exported_resource_records = export_shader_resource_registry_for_asset_roots(
        &args.asset_roots,
        args.export_resource_registry.as_ref(),
    )
    .map_err(|error| error.to_string())?;
    let resource_registry = if let Some(path) = args.resource_registry.as_deref() {
        Some(ShaderPrewarmResourceRegistryOverlay::read(path).map_err(|error| error.to_string())?)
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
                &geometry_source_descriptors,
                &shading_model_ids,
                resource_registry.as_ref(),
            )
            .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }

    let cache_dir = args
        .cache_dir
        .unwrap_or_else(|| default_shader_variant_cache_root_for_project(&args.project_root));
    let report = if args.validate_wgpu_pipelines {
        prewarm_shader_variants_with_wgpu_pipeline_validation(&manifest, &cache_dir)
    } else if args.validate_wgpu_modules {
        prewarm_shader_variants_with_wgpu_module_validation(&manifest, &cache_dir)
    } else {
        prewarm_shader_variants(&manifest, &cache_dir)
    };
    let json =
        encode_shader_prewarm_report(&report, args.pretty).map_err(|error| error.to_string())?;

    if let Some(report_path) = &args.report {
        write_shader_prewarm_report(report_path, &json).map_err(|error| error.to_string())?;
    }

    println!("{json}");
    if report.failed_count > 0 {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
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
    fs::write(export_path, json).map_err(|source| {
        ShaderPrewarmResourceRegistryError::WriteExport {
            path: export_path.to_path_buf(),
            source,
        }
    })?;
    Ok(Some(records))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::resource::{ResourceKind, ResourceRecord, ResourceState};

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
                r#"format_version = 6
uuid = "{id}"
url = "{locator}"
asset_kind = "Shader"
unit = "single"
source_hash = "{source_hash}"
"#
            ),
        )
        .unwrap();
    }
}

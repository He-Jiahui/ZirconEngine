//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary() {
    let main = include_str!("../../../../bin/zircon_shader_prewarm/main.rs");
    let args = include_str!("../../../../bin/zircon_shader_prewarm/args.rs");
    let error_owner = include_str!("../../../../bin/zircon_shader_prewarm/error.rs");
    let manifest = include_str!("../../../../bin/zircon_shader_prewarm/manifest.rs");
    let manifest_material_sources =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/material_sources.rs");
    let manifest_paths = include_str!("../../../../bin/zircon_shader_prewarm/manifest/paths.rs");
    let manifest_tests =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/tests/cases.rs");
    let manifest_asset_scan_tests =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/tests/asset_scan_errors.rs");
    let manifest_io_tests =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/tests/io.rs");
    let permutation_registry =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/permutation_registry.rs");
    let resource_registry =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let resource_registry_tests =
        include_str!("../../../../bin/manifest/resource_registry/tests/cases.rs");
    let run = include_str!("../../../../bin/zircon_shader_prewarm/run.rs");
    let _review_findings = crate::runtime_test_history_concat!(
        include_str!("../../../../../../docs/architecture/runtime-architecture-review-m0.md"),
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md")
    );
    let _render_doc = include_str!(
        "../../../../../../docs/crates/zircon_runtime/graphics/render-product-submit.md"
    );
    let _module_doc =
        include_str!("../../../../../../docs/crates/zircon_runtime/structure/module-convention.md");

    assert!(
        main.contains("mod error;"),
        "shader prewarm bin root should mount the typed error owner"
    );
    let args_tests = include_str!("../../../../bin/zircon_shader_prewarm/tests/args.rs");
    assert!(
        args.replace("\r\n", "\n")
            .contains("#[cfg(test)]\n#[path = \"tests/args.rs\"]\nmod tests;"),
        "exact current module mount in bin/zircon_shader_prewarm/args.rs"
    );
    let run_tests = include_str!("../../../../bin/zircon_shader_prewarm/tests/run.rs");
    assert!(
        run.replace("\r\n", "\n")
            .contains("#[cfg(test)]\n#[path = \"tests/run.rs\"]\nmod tests;"),
        "exact current module mount in bin/zircon_shader_prewarm/run.rs"
    );
    let permutation_registry_tests =
        include_str!("../../../../bin/manifest/tests/permutation_registry.rs");
    assert!(
        permutation_registry.replace("\r\n", "\n").contains(
            "#[cfg(test)]\n#[path = \"../../manifest/tests/permutation_registry.rs\"]\nmod tests;"
        ),
        "exact current module mount in bin/zircon_shader_prewarm/manifest/permutation_registry.rs"
    );
    for required in [
        "fn shader_prewarm_permutation_registry_read_reports_typed_read_error",
        "fn shader_prewarm_permutation_registry_read_reports_typed_parse_error",
        "fn shader_prewarm_permutation_registry_reports_typed_geometry_id_range_error",
        "ShaderPrewarmPermutationRegistryOverlay::read(&registry_path).unwrap_err()",
    ] {
        assert!(
            permutation_registry_tests.contains(required),
            "current bin/manifest/tests/permutation_registry.rs must retain {required}"
        );
    }

    for required in [
        "fn shader_prewarm_report_write_reports_typed_directory_error",
        "write_shader_prewarm_report(&report_path, \"{}\").unwrap_err()",
        "fn shader_prewarm_resource_registry_export_reports_typed_directory_error",
    ] {
        assert!(
            run_tests.contains(required),
            "current bin/zircon_shader_prewarm/tests/run.rs must retain {required}"
        );
    }

    for required in ["fn shader_prewarm_args_missing_value_reports_typed_usage_error"] {
        assert!(
            args_tests.contains(required),
            "current bin/zircon_shader_prewarm/tests/args.rs must retain {required}"
        );
    }

    for required in [
        "pub type ShaderPrewarmArgsResult<T> = std::result::Result<T, ShaderPrewarmArgsError>;",
        "pub type ShaderPrewarmAssetScanResult<T> = std::result::Result<T, ShaderPrewarmAssetScanError>;",
        "pub type ShaderPrewarmManifestResult<T> = std::result::Result<T, ShaderPrewarmManifestError>;",
        "pub type ShaderPrewarmPermutationRegistryResult<T> =",
        "pub type ShaderPrewarmReportResult<T> = std::result::Result<T, ShaderPrewarmReportError>;",
        "pub type ShaderPrewarmResourceRegistryResult<T> =",
        "pub enum ShaderPrewarmArgsError",
        "pub enum ShaderPrewarmAssetScanError",
        "pub enum ShaderPrewarmManifestError",
        "pub enum ShaderPrewarmPermutationRegistryError",
        "pub enum ShaderPrewarmReportError",
        "pub enum ShaderPrewarmResourceRegistryError",
        "Usage(String)",
        "ReadAssetRoot {",
        "ReadAssetRootEntry {",
        "LoadShaderMetadata {",
        "ReadZShader {",
        "ParseZShader {",
        "ReadWgsl {",
        "ReadZMaterial {",
        "ParseZMaterial {",
        "EmptyShaderSource {",
        "ReadShaderPackage {",
        "ReadShaderPackageEntry {",
        "ShaderSourceOutsidePackageDir {",
        "Read {",
        "source: std::io::Error",
        "Parse {",
        "source: serde_json::Error",
        "UnsupportedSchema { actual: u32, expected: u32 }",
        "InvalidToken(#[from] ShaderPrewarmArgsError)",
        "GeometrySourceIdBelowPluginRange {",
        "ShadingModelIdBelowPluginRange {",
        "ReportEncode {",
        "CreateReportDirectory {",
        "WriteReport {",
        "MissingRecordsArray {",
        "DecodeRecords {",
        "ReadRoot {",
        "LoadMetadata {",
        "DuplicateRecordId {",
        "DuplicateLocator {",
        "EncodeExport {",
        "CreateExportDirectory {",
        "WriteExport {",
    ] {
        assert!(
            error_owner.contains(required),
            "shader prewarm typed-error owner should contain `{required}`"
        );
    }

    for required in [
        "pub fn parse(",
        ") -> ShaderPrewarmArgsResult<Option<ShaderPrewarmArgs>>",
        "ShaderPrewarmArgsError::Usage",
        ") -> ShaderPrewarmArgsResult<String>",
        ") -> ShaderPrewarmArgsResult<PathBuf>",
        "fn parse_quality_tier(value: &str) -> ShaderPrewarmArgsResult<Vec<ShaderQualityTier>>",
        "fn parse_geometry_source(value: &str) -> ShaderPrewarmArgsResult<Vec<GeometrySourceId>>",
        "fn parse_geometry_source_id(value: &str) -> ShaderPrewarmArgsResult<(String, GeometrySourceId)>",
        ") -> ShaderPrewarmArgsResult<String>",
        "fn parse_shading_model_id(value: &str) -> ShaderPrewarmArgsResult<(String, ShadingModelId)>",
        ") -> ShaderPrewarmArgsResult<BTreeMap<String, ShadingModelId>>",
        ") -> ShaderPrewarmArgsResult<BTreeMap<String, GeometrySourceId>>",
    ] {
        assert!(
            args.contains(required),
            "shader prewarm args typed-error path should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<Option<ShaderPrewarmArgs>, String>",
        "Result<PathBuf, String>",
        "Result<String, String>",
        "Result<Vec<ShaderQualityTier>, String>",
        "Result<Vec<GeometrySourceId>, String>",
        "Result<(String, GeometrySourceId), String>",
        "Result<(String, ShadingModelId), String>",
        "Result<BTreeMap<String, ShadingModelId>, String>",
        "Result<BTreeMap<String, GeometrySourceId>, String>",
        ".ok_or_else(|| usage(",
        ".map_err(|_| usage(",
        "return Err(usage(",
    ] {
        assert!(
            !production_section(args).contains(forbidden),
            "shader prewarm args should not keep lossy String-error branch `{forbidden}`"
        );
    }

    assert!(
        run.contains("parse(args).map_err(|error| error.to_string())?"),
        "shader prewarm run.rs should remain the CLI/run display boundary for args typed errors"
    );
    for required in [
        "finish_shader_prewarm_report(&report, args.report.as_deref(), args.pretty)",
        "fn finish_shader_prewarm_report(",
        "report: &ShaderVariantPrewarmReport,",
        "report_path: Option<&Path>,",
        "pretty: bool,",
    ] {
        assert!(
            run.contains(required),
            "current bin/zircon_shader_prewarm/run.rs must retain {required}"
        );
    }
    for required in [
        "use super::error::{",
        "ShaderPrewarmReportError",
        "ShaderPrewarmReportResult",
        "fn encode_shader_prewarm_report(",
        ") -> ShaderPrewarmReportResult<String>",
        "ShaderPrewarmReportError::ReportEncode",
        "fn write_shader_prewarm_report(",
        ") -> ShaderPrewarmReportResult<()>",
        "ShaderPrewarmReportError::CreateReportDirectory",
        "ShaderPrewarmReportError::WriteReport",
        "encode_shader_prewarm_report(report, pretty).map_err(|error| error.to_string())?",
        "write_shader_prewarm_report(report_path, &json).map_err(|error| error.to_string())?",
    ] {
        assert!(
            run.contains(required),
            "shader prewarm report output typed-error path should contain `{required}`"
        );
    }
    for forbidden in [
        "format!(\"failed to encode shader prewarm report: {error}\")",
        "\"failed to create shader prewarm report directory {}: {error}\"",
        "\"failed to write shader prewarm report {}: {error}\"",
    ] {
        assert!(
            !production_section(run).contains(forbidden),
            "shader prewarm report output should not keep lossy String-error branch `{forbidden}`"
        );
    }
    for required in [
        "ShaderPrewarmManifestError",
        "ShaderPrewarmManifestResult",
        "pub fn read_manifest(path: &Path) -> ShaderPrewarmManifestResult<ShaderVariantPrewarmManifest>",
        "ShaderPrewarmManifestError::Read",
        "ShaderPrewarmManifestError::Parse",
        ") -> ShaderPrewarmManifestResult<ShaderVariantPrewarmManifest>",
        "fn shader_prewarm_read_manifest_reports_typed_read_error",
        "fn shader_prewarm_read_manifest_reports_typed_parse_error",
        "read_manifest(&missing_path).unwrap_err()",
        "read_manifest(&manifest_path).unwrap_err()",
        "ShaderPrewarmManifestError::UnsupportedSchema",
        "fn shader_prewarm_merge_manifest_reports_typed_schema_error",
        "merge_manifests(stale_manifest, valid_manifest).unwrap_err()",
    ] {
        assert!(
            manifest.contains(required)
                || manifest_tests.contains(required)
                || manifest_io_tests.contains(required),
            "shader prewarm manifest merge typed-error path should contain `{required}`"
        );
    }
    for forbidden in [
        "pub fn read_manifest(path: &Path) -> Result<ShaderVariantPrewarmManifest, String>",
        "\"failed to read shader prewarm manifest {}: {error}\"",
        "\"failed to parse shader prewarm manifest {}: {error}\"",
        "pub fn merge_manifests(\n    mut base: ShaderVariantPrewarmManifest,\n    extra: ShaderVariantPrewarmManifest,\n) -> Result<ShaderVariantPrewarmManifest, String>",
        "\"shader prewarm manifest schema {} is not supported; expected {}\"",
    ] {
        assert!(
            !production_section(manifest).contains(forbidden),
            "shader prewarm manifest merge should not keep lossy String-error branch `{forbidden}`"
        );
    }
    assert!(
        run.contains("read_manifest(path).map_err(|error| error.to_string())?")
            && run.contains("merge_manifests(manifest, manifest_from_file)")
            && run.contains(".map_err(|error| error.to_string())?"),
        "shader prewarm run.rs should remain the CLI/run display boundary for manifest read typed errors"
    );
    assert!(
        run.contains("merge_manifests(") && run.contains(".map_err(|error| error.to_string())?"),
        "shader prewarm run.rs should remain the CLI/run display boundary for manifest merge typed errors"
    );
    let asset_inventory =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/asset_inventory.rs");
    let asset_inventory_traversal =
        include_str!("../../../../bin/manifest/asset_inventory/traversal.rs");
    assert!(
        manifest
            .replace("\r\n", "\n")
            .contains("#[path = \"manifest/asset_inventory.rs\"]\nmod asset_inventory;"),
        "exact current module mount in bin/zircon_shader_prewarm/manifest.rs"
    );
    assert!(
        asset_inventory
            .replace("\r\n", "\n")
            .contains("#[path = \"../../manifest/asset_inventory/traversal.rs\"]\nmod traversal;"),
        "exact current module mount in bin/zircon_shader_prewarm/manifest/asset_inventory.rs"
    );
    for required in [
        "ShaderPrewarmAssetScanError::ReadAssetRoot",
        "ShaderPrewarmAssetScanError::LoadShaderMetadata",
        "AssetMetaDocument::load(meta_path).map_err(|source|",
        "inventory_text_read_error(path, source)",
        "collect_file_paths(",
    ] {
        assert!(
            asset_inventory.contains(required),
            "current bin/zircon_shader_prewarm/manifest/asset_inventory.rs must retain {required}"
        );
    }
    for required in [
        "ShaderPrewarmAssetScanError::ReadAssetRootEntry",
        "ShaderPrewarmAssetScanError::ReadWgsl",
        "fn inventory_text_read_error(",
        "source: std::io::Error,",
    ] {
        assert!(
            asset_inventory_traversal.contains(required),
            "current bin/manifest/asset_inventory/traversal.rs must retain {required}"
        );
    }
    for required in [
        "ShaderPrewarmAssetScanResult",
        "ShaderPrewarmAssetScanError::ParseZShader",
        "ShaderPrewarmAssetScanError::ParseZMaterial",
        "ShaderPrewarmAssetScanError::EmptyShaderSource",
        "ShaderPrewarmAssetScanError::ShaderSourceOutsidePackageDir",
        "fn shader_prewarm_asset_root_scan_reports_typed_read_root_error",
        "fn shader_prewarm_asset_root_scan_reports_typed_zshader_parse_error",
        "fn shader_prewarm_asset_root_scan_reports_typed_empty_wgsl_error",
        "fn shader_prewarm_asset_root_scan_reports_typed_zmaterial_parse_error",
    ] {
        assert!(
            manifest.contains(required)
                || manifest_material_sources.contains(required)
                || manifest_paths.contains(required)
                || manifest_tests.contains(required)
                || manifest_asset_scan_tests.contains(required)
                || error_owner.contains(required),
            "shader prewarm asset-root scan typed-error path should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<ShaderVariantPrewarmManifest, String>",
        "Result<Option<ShaderPrewarmSource>, String>",
        "Result<ShaderPrewarmSource, String>",
        "Result<MaterialPrewarmSource, String>",
        "Result<Vec<PathBuf>, String>",
        "Result<(), String>",
        "\"failed to read asset root {}: {error}\"",
        "\"failed to read asset root {} entry: {error}\"",
        "\"failed to load shader asset metadata {}: {error}\"",
        "\"failed to read zshader {}: {error}\"",
        "\"failed to parse zshader {}: {error}\"",
        "\"failed to read WGSL {}: {error}\"",
        "\"failed to read zmaterial {}: {error}\"",
        "\"failed to parse zmaterial {}: {error}\"",
        "\"shader source {} has no runtime WGSL payload\"",
        "\"shader source {} is outside package dir {}: {error}\"",
    ] {
        assert!(
            !production_section(manifest).contains(forbidden)
                && !production_section(asset_inventory).contains(forbidden)
                && !production_section(asset_inventory_traversal).contains(forbidden),
            "shader prewarm asset-root scan should not keep lossy String-error branch `{forbidden}`"
        );
    }
    assert!(
        run.contains("asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs(")
            && run.contains(".map_err(|error| error.to_string())?"),
        "shader prewarm run.rs should remain the CLI/run display boundary for asset-root scan typed errors"
    );
    for required in [
        "ShaderPrewarmPermutationRegistryResult",
        "ShaderPrewarmPermutationRegistryError::Read",
        "ShaderPrewarmPermutationRegistryError::Parse",
        "ShaderPrewarmPermutationRegistryError::GeometrySourceIdBelowPluginRange",
        "ShaderPrewarmPermutationRegistryError::DuplicateGeometrySourceToken",
    ] {
        assert!(
            permutation_registry.contains(required),
            "shader prewarm permutation registry typed-error path should contain `{required}`"
        );
    }
    for forbidden in [
        "pub(crate) fn read(path: &Path) -> Result<Self, String>",
        "\"failed to read shader prewarm permutation registry {}: {error}\"",
        "\"failed to parse shader prewarm permutation registry {}: {error}\"",
        "fn geometry_source_id_from_registry(id: u8, path: &Path) -> Result<GeometrySourceId, String>",
        "fn shading_model_id_from_registry(id: u8, path: &Path) -> Result<ShadingModelId, String>",
    ] {
        assert!(
            !production_section(permutation_registry).contains(forbidden),
            "shader prewarm permutation registry should not keep lossy String-error branch `{forbidden}`"
        );
    }
    assert!(
        run.contains("ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)")
            && run.contains(".map_err(|error| error.to_string())?")
            && run.contains("registry_overlay")
            && run.contains(".merge_into("),
        "shader prewarm run.rs should remain the CLI/run display boundary for permutation registry typed errors"
    );
    for required in [
        "ShaderPrewarmResourceRegistryResult",
        "ShaderPrewarmResourceRegistryError::Read",
        "ShaderPrewarmResourceRegistryError::Parse",
        "ShaderPrewarmResourceRegistryError::DecodeRecords",
        "ShaderPrewarmResourceRegistryError::ReadRoot",
        "ShaderPrewarmResourceRegistryError::LoadMetadata",
        "ShaderPrewarmResourceRegistryError::DuplicateRecordId",
        "ShaderPrewarmResourceRegistryError::DuplicateLocator",
        "fn shader_prewarm_resource_registry_read_reports_typed_read_error",
        "fn shader_prewarm_resource_registry_read_reports_typed_parse_error",
        "fn shader_prewarm_resource_registry_read_reports_typed_decode_error",
        "ShaderPrewarmResourceRegistryOverlay::read(&missing_path).unwrap_err()",
    ] {
        assert!(
            resource_registry.contains(required) || resource_registry_tests.contains(required),
            "shader prewarm resource registry typed-error path should contain `{required}`"
        );
    }
    for forbidden in [
        "pub(crate) fn read(path: &Path) -> Result<Self, String>",
        "Result<Vec<ResourceRecord>, String>",
        "Result<(), String>",
        "\"failed to read shader prewarm resource registry {}: {error}\"",
        "\"failed to parse shader prewarm resource registry {}: {error}\"",
        "\"failed to decode shader prewarm resource records {}: {error}\"",
        "\"failed to read shader resource registry root {}: {error}\"",
        "\"failed to load shader resource registry metadata {}: {error}\"",
    ] {
        assert!(
            !production_section(resource_registry).contains(forbidden),
            "shader prewarm resource registry should not keep lossy String-error branch `{forbidden}`"
        );
    }
    for required in [
        "ShaderPrewarmResourceRegistryError::EncodeExport",
        "ShaderPrewarmResourceRegistryError::CreateExportDirectory",
        "ShaderPrewarmResourceRegistryError::WriteExport",
        "export_shader_resource_registry_for_asset_roots(",
        ".map_err(|error| error.to_string())?",
        "ShaderPrewarmResourceRegistryOverlay::read(path).map_err(|error| error.to_string())?",
    ] {
        assert!(
            run.contains(required),
            "shader prewarm resource registry run/export path should contain `{required}`"
        );
    }
    for forbidden in [
        "format!(\"failed to encode shader resource registry: {error}\")",
        "\"failed to create shader resource registry directory {}: {error}\"",
        "\"failed to write shader resource registry {}: {error}\"",
    ] {
        assert!(
            !production_section(run).contains(forbidden),
            "shader prewarm resource registry export should not keep lossy String-error branch `{forbidden}`"
        );
    }
}

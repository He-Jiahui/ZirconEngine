#[test]
fn review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary() {
    let main = include_str!("../../../../../bin/zircon_shader_prewarm/main.rs");
    let args = include_str!("../../../../../bin/zircon_shader_prewarm/args.rs");
    let error_owner = include_str!("../../../../../bin/zircon_shader_prewarm/error.rs");
    let manifest = include_str!("../../../../../bin/zircon_shader_prewarm/manifest.rs");
    let manifest_material_sources =
        include_str!("../../../../../bin/zircon_shader_prewarm/manifest/material_sources.rs");
    let manifest_paths = include_str!("../../../../../bin/zircon_shader_prewarm/manifest/paths.rs");
    let manifest_tests = include_str!("../../../../../bin/zircon_shader_prewarm/manifest/tests.rs");
    let manifest_asset_scan_tests = include_str!(
        "../../../../../bin/zircon_shader_prewarm/manifest/tests/asset_scan_errors.rs"
    );
    let manifest_io_tests =
        include_str!("../../../../../bin/zircon_shader_prewarm/manifest/tests/io.rs");
    let permutation_registry =
        include_str!("../../../../../bin/zircon_shader_prewarm/manifest/permutation_registry.rs");
    let resource_registry =
        include_str!("../../../../../bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let resource_registry_tests = include_str!(
        "../../../../../bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs"
    );
    let run = include_str!("../../../../../bin/zircon_shader_prewarm/run.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md")
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let render_doc =
        include_str!("../../../../../../../docs/zircon_runtime/graphics/render-product-submit.md");
    let module_doc =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");

    assert!(
        main.contains("mod error;"),
        "shader prewarm bin root should mount the typed error owner"
    );
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
        "fn shader_prewarm_args_missing_value_reports_typed_usage_error",
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
            !args.contains(forbidden),
            "shader prewarm args should not keep lossy String-error branch `{forbidden}`"
        );
    }

    assert!(
        run.contains("parse(args).map_err(|error| error.to_string())?"),
        "shader prewarm run.rs should remain the CLI/run display boundary for args typed errors"
    );
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
        "encode_shader_prewarm_report(&report, args.pretty).map_err(|error| error.to_string())?",
        "write_shader_prewarm_report(report_path, &json).map_err(|error| error.to_string())?",
        "fn shader_prewarm_report_write_reports_typed_directory_error",
        "write_shader_prewarm_report(&report_path, \"{}\").unwrap_err()",
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
            !run.contains(forbidden),
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
            !manifest.contains(forbidden),
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
    for required in [
        "ShaderPrewarmAssetScanResult",
        "ShaderPrewarmAssetScanError::ReadAssetRoot",
        "ShaderPrewarmAssetScanError::ReadAssetRootEntry",
        "ShaderPrewarmAssetScanError::LoadShaderMetadata",
        "ShaderPrewarmAssetScanError::ParseZShader",
        "ShaderPrewarmAssetScanError::ReadWgsl",
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
            !manifest.contains(forbidden),
            "shader prewarm asset-root scan should not keep lossy String-error branch `{forbidden}`"
        );
    }
    assert!(
        run.contains("asset_root_manifest_with_resource_registry_revisions(")
            && run.contains(".map_err(|error| error.to_string())?"),
        "shader prewarm run.rs should remain the CLI/run display boundary for asset-root scan typed errors"
    );
    for required in [
        "ShaderPrewarmPermutationRegistryResult",
        "ShaderPrewarmPermutationRegistryError::Read",
        "ShaderPrewarmPermutationRegistryError::Parse",
        "ShaderPrewarmPermutationRegistryError::GeometrySourceIdBelowPluginRange",
        "ShaderPrewarmPermutationRegistryError::DuplicateGeometrySourceToken",
        "fn shader_prewarm_permutation_registry_read_reports_typed_read_error",
        "fn shader_prewarm_permutation_registry_read_reports_typed_parse_error",
        "fn shader_prewarm_permutation_registry_reports_typed_geometry_id_range_error",
        "ShaderPrewarmPermutationRegistryOverlay::read(&registry_path).unwrap_err()",
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
            !permutation_registry.contains(forbidden),
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
            !resource_registry.contains(forbidden),
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
        "fn shader_prewarm_resource_registry_export_reports_typed_directory_error",
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
            !run.contains(forbidden),
            "shader prewarm resource registry export should not keep lossy String-error branch `{forbidden}`"
        );
    }
}

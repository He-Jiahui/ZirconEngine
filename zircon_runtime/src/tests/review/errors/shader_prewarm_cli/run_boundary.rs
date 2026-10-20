//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary() {
    let args = include_str!("../../../../bin/zircon_shader_prewarm/args.rs");
    let error_owner = include_str!("../../../../bin/zircon_shader_prewarm/error.rs");
    let main = include_str!("../../../../bin/zircon_shader_prewarm/main.rs");
    let manifest = include_str!("../../../../bin/zircon_shader_prewarm/manifest.rs");
    let paths = include_str!("../../../../bin/zircon_shader_prewarm/manifest/paths.rs");
    let permutation_registry =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/permutation_registry.rs");
    let resource_registry =
        include_str!("../../../../bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let run = include_str!("../../../../bin/zircon_shader_prewarm/run.rs");
    let _render_doc = include_str!(
        "../../../../../../docs/crates/zircon_runtime/graphics/render-product-submit.md"
    );
    let _module_doc =
        include_str!("../../../../../../docs/crates/zircon_runtime/structure/module-convention.md");

    for required in [
        "pub type ShaderPrewarmArgsResult",
        "pub type ShaderPrewarmAssetScanResult",
        "pub type ShaderPrewarmManifestResult",
        "pub type ShaderPrewarmPermutationRegistryResult",
        "pub type ShaderPrewarmReportResult",
        "pub type ShaderPrewarmResourceRegistryResult",
        "pub enum ShaderPrewarmArgsError",
        "pub enum ShaderPrewarmAssetScanError",
        "pub enum ShaderPrewarmManifestError",
        "pub enum ShaderPrewarmPermutationRegistryError",
        "pub enum ShaderPrewarmReportError",
        "pub enum ShaderPrewarmResourceRegistryError",
    ] {
        assert!(
            error_owner.contains(required),
            "shader prewarm final sweep should keep typed owner anchor `{required}`"
        );
    }

    for (label, source) in [
        ("args", args),
        ("manifest", manifest),
        ("manifest paths", paths),
        ("permutation registry", permutation_registry),
        ("resource registry", resource_registry),
    ] {
        for forbidden in [
            "Result<ExitCode, String>",
            "Result<(), String>",
            "Result<String, String>",
            "Result<Vec<PathBuf>, String>",
            "Result<ShaderVariantPrewarmManifest, String>",
            "Result<ShaderPrewarmSource, String>",
            "Result<MaterialPrewarmSource, String>",
            "Err(format!(",
            ".map_err(|error| error.to_string())",
        ] {
            assert!(
                !source.contains(forbidden),
                "shader prewarm {label} should not keep internal String-error branch `{forbidden}`"
            );
        }
    }

    assert!(
        run.contains(
            "pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<ExitCode, String>"
        ) && run.matches(".map_err(|error| error.to_string())?").count() >= 8
            && main.contains("Err(error) => {")
            && main.contains("eprintln!(\"{error}\");"),
        "shader prewarm should keep String diagnostics only at run.rs/main.rs display boundaries"
    );
    let projection = run
        .split("asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs(")
        .nth(1)
        .and_then(|source| source.split(".map_err(|error| error.to_string())?").next())
        .expect("typed inventory projection reaches only the CLI display boundary");
    let projection = projection.split_whitespace().collect::<String>();
    assert!(projection.contains("asset_root,inventory,"));
    assert!(projection.contains("resource_registry.as_ref(),has_external_permutation_inputs,"));
    assert!(manifest.contains(") -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest>"));
    for required in [
        "parse(args).map_err(|error| error.to_string())?",
        "ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)",
        "read_manifest(path).map_err(|error| error.to_string())?",
        "asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs(",
        "ShaderPrewarmResourceRegistryOverlay::read(path).map_err(|error| error.to_string())?",
    ] {
        assert!(
            run.contains(required),
            "shader prewarm final sweep should keep CLI display boundary anchor `{required}`"
        );
    }
    let (run_body, finish_report) = run
        .split_once("fn finish_shader_prewarm_report(")
        .expect("both report paths must delegate to the real CLI report owner");
    assert_eq!(
        run_body
            .matches("finish_shader_prewarm_report(&report, args.report.as_deref(), args.pretty)")
            .count(),
        2,
        "preflight rejection and completed prewarm must share the report boundary"
    );
    let finish_report = finish_report
        .split_once("/// Local inventory")
        .expect("the report helper must end before inventory policy")
        .0;
    for required in [
        ") -> Result<ExitCode, String>",
        "encode_shader_prewarm_report(report, pretty).map_err(|error| error.to_string())?",
        "write_shader_prewarm_report(report_path, &json).map_err(|error| error.to_string())?",
        "if report.failed_count > 0 || report.preflight_error.is_some()",
        "Ok(ExitCode::from(2))",
        "Ok(ExitCode::SUCCESS)",
    ] {
        assert!(finish_report.contains(required), "{required}");
    }
    for required in [
        ") -> ShaderPrewarmReportResult<String>",
        "result.map_err(|source| ShaderPrewarmReportError::ReportEncode { source })",
        "fn write_shader_prewarm_report(report_path: &Path, json: &str) -> ShaderPrewarmReportResult<()>",
        "ShaderPrewarmReportError::WriteReport {",
    ] {
        assert!(run.contains(required), "{required}");
    }
}

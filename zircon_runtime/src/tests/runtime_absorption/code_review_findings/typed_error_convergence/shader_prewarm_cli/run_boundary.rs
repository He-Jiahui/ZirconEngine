#[test]
fn review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary() {
    let args = include_str!("../../../../../bin/zircon_shader_prewarm/args.rs");
    let error_owner = include_str!("../../../../../bin/zircon_shader_prewarm/error.rs");
    let main = include_str!("../../../../../bin/zircon_shader_prewarm/main.rs");
    let manifest = include_str!("../../../../../bin/zircon_shader_prewarm/manifest.rs");
    let paths = include_str!("../../../../../bin/zircon_shader_prewarm/manifest/paths.rs");
    let permutation_registry =
        include_str!("../../../../../bin/zircon_shader_prewarm/manifest/permutation_registry.rs");
    let resource_registry =
        include_str!("../../../../../bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let run = include_str!("../../../../../bin/zircon_shader_prewarm/run.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
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
    for required in [
        "parse(args).map_err(|error| error.to_string())?",
        "ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)",
        "read_manifest(path).map_err(|error| error.to_string())?",
        "asset_root_manifest_with_resource_registry_revisions(",
        "encode_shader_prewarm_report(&report, args.pretty).map_err(|error| error.to_string())?",
        "write_shader_prewarm_report(report_path, &json).map_err(|error| error.to_string())?",
        "ShaderPrewarmResourceRegistryOverlay::read(path).map_err(|error| error.to_string())?",
    ] {
        assert!(
            run.contains(required),
            "shader prewarm final sweep should keep CLI display boundary anchor `{required}`"
        );
    }
}

use super::runtime_crates::IMPORTER_RUNTIME_CRATES;

#[test]
fn review_d13_importer_runtime_manifests_use_sdk_builder() {
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
    );
    let sdk_importer_manifest = include_str!(
        "../../../../../../../zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs"
    );

    assert_eq!(
        IMPORTER_RUNTIME_CRATES.len(),
        12,
        "D13 importer manifest builder guard should cover every first-party importer runtime owner"
    );

    for required in [
        "pub struct ImporterRuntimeManifestBuilder",
        "pub fn importer_runtime_supported_targets() -> [RuntimeTargetMode; 2]",
        "pub fn importer_runtime_supported_platforms() -> [ExportTargetPlatform; 3]",
        "pub const NATIVE_DESCRIPTOR_SYMBOL_V3",
        "pub const NATIVE_ABI_VERSION_V3",
        "PluginModuleManifest::runtime",
        "PluginModuleManifest::native",
        "PluginDistributionManifest",
        "build_package_manifest",
    ] {
        assert!(
            sdk_importer_manifest.contains(required),
            "plugin SDK importer manifest helper should own `{required}`"
        );
    }

    for (label, source, _) in IMPORTER_RUNTIME_CRATES {
        for required in [
            "ImporterRuntimeManifestBuilder",
            "importer_runtime_supported_targets()",
            "importer_runtime_supported_platforms()",
            "importer_manifest_builder().runtime_module_manifest()",
            "importer_manifest_builder().dist_module_manifest()",
            ".build_package_manifest(descriptor)",
        ] {
            assert!(
                source.contains(required),
                "{label} should route importer manifest boilerplate through SDK builder `{required}`"
            );
        }

        for stale in [
            "PluginDistributionManifest",
            "ExportPackagingStrategy",
            "NATIVE_DESCRIPTOR_SYMBOL_V3",
            "NATIVE_ABI_VERSION_V3",
            "DIST_ENGINE_COMPAT",
            "PluginModuleManifest::runtime",
            "PluginModuleManifest::native",
            "with_distribution(",
            "default_packaging.push",
        ] {
            assert!(
                !source.contains(stale),
                "{label} should not keep hand-written importer manifest boilerplate `{stale}`"
            );
        }
    }

    let d13_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D13 |"))
        .expect("D13 review finding row should exist");
    for required in [
        "selection/helper 导出与 targets/platforms/module/dist-module manifest 样板已由 plugin SDK 收敛",
        "ImporterRuntimeManifestBuilder",
        "d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred",
        "review_d13_importer_runtime_manifests_use_sdk_builder",
    ] {
        assert!(
            review_findings.contains(required),
            "D13 numbered review evidence should record importer runtime manifest builder convergence anchor `{required}`"
        );
    }
    assert!(
        !d13_row.contains("targets/platforms/module/dist-module 样板仍是后续 builder/parity"),
        "D13 row should not keep stale importer manifest builder follow-up text"
    );

    assert!(
        review_findings.contains("D13 importer runtime manifest builder convergence")
            && review_findings.contains("review_d13_importer_runtime_manifests_use_sdk_builder"),
        "D13 numbered output should own the concrete importer manifest evidence"
    );
}

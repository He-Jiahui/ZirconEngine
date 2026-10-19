//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use super::runtime_crates::IMPORTER_RUNTIME_CRATES;
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;

#[test]
fn review_d13_importer_runtime_manifests_use_sdk_builder() {
    let review_findings = crate::runtime_test_history_concat!(
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../docs/architecture/runtime-architecture-review-m0.md")
    );
    let sdk_importer_manifest = include_str!(
        "../../../../../../zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs"
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

    let declarations = [
        ("asset_importers/audio", "AUDIO_ASSET_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/asset_importers/audio/runtime/src/capability.rs")),
        ("asset_importers/data", "DATA_ASSET_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/asset_importers/data/runtime/src/capability.rs")),
        ("asset_importers/model", "MODEL_ASSET_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/asset_importers/model/runtime/src/capability.rs")),
        ("asset_importers/shader", "SHADER_ASSET_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/asset_importers/shader/runtime/src/capability.rs")),
        ("asset_importers/texture", "TEXTURE_ASSET_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/asset_importers/texture/runtime/src/capability.rs")),
        ("audio_importer", "AUDIO_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/audio_importer/runtime/src/capability.rs")),
        ("gltf_importer", "GLTF_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/gltf_importer/runtime/src/capability.rs")),
        ("obj_importer", "OBJ_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/obj_importer/runtime/src/capability.rs")),
        ("opus_importer", "OPUS_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/opus_importer/runtime/src/capability.rs")),
        ("shader_wgsl_importer", "SHADER_WGSL_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/shader_wgsl_importer/runtime/src/capability.rs")),
        ("texture_importer", "TEXTURE_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/texture_importer/runtime/src/capability.rs")),
        ("ui_document_importer", "UI_DOCUMENT_IMPORTER_DECLARATION", include_str!("../../../../../../zircon_plugins/ui_document_importer/runtime/src/capability.rs")),
    ];
    assert_eq!(declarations.len(), IMPORTER_RUNTIME_CRATES.len());
    for ((label, source, _), (declaration_label, declaration, capability)) in
        IMPORTER_RUNTIME_CRATES.iter().zip(declarations)
    {
        assert_eq!(*label, declaration_label);
        let compact: String = source.split_whitespace().collect();
        for suffix in [
            ".runtime_declaration(RUNTIME_CRATE_NAME)",
            ".target_modes()",
            ".supported_platforms()",
            ".module_name()",
        ] {
            assert!(
                compact.contains(&format!("{declaration}{suffix}")),
                "{label} must delegate {suffix} to its own declaration"
            );
        }
        for required in [
            "zircon_plugin_sdk::declare_plugin!",
            "targets: [client_runtime, editor_host]",
            "platforms: [windows, linux, macos]",
        ] {
            assert!(
                capability.contains(required),
                "{label} declaration missing {required}"
            );
        }
        assert!(capability.contains(declaration));
    }

    for (label, source, _) in IMPORTER_RUNTIME_CRATES {
        for required in [
            "ImporterRuntimeManifestBuilder",
            ".target_modes()",
            ".supported_platforms()",
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

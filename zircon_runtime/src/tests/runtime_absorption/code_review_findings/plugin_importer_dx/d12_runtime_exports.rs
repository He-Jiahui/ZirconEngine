const D12_TRAIT_BACKED_RUNTIME_CRATES: &[(&str, &str, &str, &str)] = &[
    (
        "ai",
        "AiRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/ai/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/ai/runtime/Cargo.toml"),
    ),
    (
        "animation",
        "AnimationRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/animation/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/animation/runtime/Cargo.toml"),
    ),
    (
        "hybrid_gi",
        "HybridGiRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/hybrid_gi/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/hybrid_gi/runtime/Cargo.toml"),
    ),
    (
        "navigation",
        "NavigationRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/navigation/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/navigation/runtime/Cargo.toml"),
    ),
    (
        "net",
        "NetRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/net/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/net/runtime/Cargo.toml"),
    ),
    (
        "particles",
        "ParticlesRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/particles/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/particles/runtime/Cargo.toml"),
    ),
    (
        "physics",
        "PhysicsRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/physics/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/physics/runtime/Cargo.toml"),
    ),
    (
        "prefab_tools",
        "PrefabToolsRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/prefab_tools/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/prefab_tools/runtime/Cargo.toml"),
    ),
    (
        "rendering",
        "RenderingRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/rendering/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/rendering/runtime/Cargo.toml"),
    ),
    (
        "solari",
        "SolariRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/solari/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/solari/runtime/Cargo.toml"),
    ),
    (
        "terrain",
        "TerrainRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/terrain/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/terrain/runtime/Cargo.toml"),
    ),
    (
        "texture",
        "TextureRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/texture/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/texture/runtime/Cargo.toml"),
    ),
    (
        "tilemap_2d",
        "Tilemap2dRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/tilemap_2d/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/tilemap_2d/runtime/Cargo.toml"),
    ),
    (
        "virtual_geometry",
        "VirtualGeometryRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/virtual_geometry/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/virtual_geometry/runtime/Cargo.toml"),
    ),
    (
        "zr_vm_language",
        "ZrVmLanguageRuntimePlugin",
        include_str!("../../../../../../zircon_plugins/zr_vm_language/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/zr_vm_language/runtime/Cargo.toml"),
    ),
];

#[test]
fn review_d12_runtime_helper_exports_use_sdk_macro() {
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
    );
    let sdk_exports =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/runtime_exports.rs");

    assert_eq!(
        D12_TRAIT_BACKED_RUNTIME_CRATES.len(),
        15,
        "D12 runtime export helper guard should cover every first-party trait-backed runtime root"
    );

    for &(label, plugin_type, plugin_source, cargo_toml) in D12_TRAIT_BACKED_RUNTIME_CRATES {
        let macro_call = format!("zircon_plugin_sdk::runtime_plugin_exports!({plugin_type});");
        assert!(
            plugin_source.contains("impl RuntimePlugin for")
                || plugin_source.contains("impl zircon_runtime::plugin::RuntimePlugin for"),
            "{label} should keep RuntimePlugin as the runtime behavior owner"
        );
        assert!(
            plugin_source.contains(&macro_call),
            "{label} should export standard helpers through SDK macro `{macro_call}`"
        );
        assert!(
            cargo_toml
                .contains("zircon_plugin_sdk = { workspace = true, features = [\"runtime\"] }"),
            "{label} should inherit the SDK runtime dependency from the plugin workspace"
        );

        for stale in [
            "pub fn runtime_plugin()",
            "pub fn package_manifest()",
            "pub fn runtime_selection()",
            "pub fn plugin_registration()",
            "RuntimePlugin::project_selection(&runtime_plugin())",
            "RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())",
        ] {
            assert!(
                !plugin_source.contains(stale),
                "{label} should not keep hand-written runtime helper block `{stale}`"
            );
        }
    }

    for required in [
        "macro_rules! runtime_plugin_exports",
        "pub fn runtime_plugin() -> $plugin_ty",
        "pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest",
        "pub fn runtime_selection(\n        ) -> zircon_runtime::core::framework::project::ProjectPluginSelection",
        "pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport",
        "zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())",
        "zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())",
        "zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())",
    ] {
        assert!(
            sdk_exports.contains(required),
            "plugin SDK runtime export macro should own generated helper `{required}`"
        );
    }

    let d12_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D12 |"))
        .expect("D12 review finding row should exist");
    for required in [
        "runtime helper export macro rollout",
        "15 个 first-party trait-backed runtime roots",
        "zircon_plugin_sdk::runtime_plugin_exports!",
        "plugins_12_runtime_export_macro_rollout_check_passed",
        "review_d12_runtime_helper_exports_use_sdk_macro",
    ] {
        assert!(
            review_findings.contains(required),
            "D12 numbered review evidence should record runtime helper export macro convergence anchor `{required}`"
        );
    }
    assert!(
        !d12_row.contains("6 个转发自由函数即便已 `impl RuntimePlugin` 仍每插件手抄"),
        "D12 row should not keep the stale copied-helper problem as current state"
    );

    assert!(
        review_findings.contains("D12 runtime helper export macro rollout")
            && review_findings.contains("review_d12_runtime_helper_exports_use_sdk_macro"),
        "D12 numbered output should own the concrete runtime export macro evidence"
    );
}

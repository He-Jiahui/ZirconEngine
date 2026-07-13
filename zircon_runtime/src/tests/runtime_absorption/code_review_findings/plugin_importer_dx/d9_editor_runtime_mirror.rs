const D9_EDITOR_RUNTIME_MIRROR_CRATES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "animation",
        include_str!("../../../../../../zircon_plugins/animation/editor/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/animation/editor/src/tests.rs"),
        include_str!("../../../../../../zircon_plugins/animation/editor/Cargo.toml"),
        "zircon_plugin_animation_runtime::ANIMATION_RUNTIME_CAPABILITY",
    ),
    (
        "physics",
        include_str!("../../../../../../zircon_plugins/physics/editor/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/physics/editor/src/tests.rs"),
        include_str!("../../../../../../zircon_plugins/physics/editor/Cargo.toml"),
        "zircon_plugin_physics_runtime::PHYSICS_RUNTIME_CAPABILITY",
    ),
    (
        "net",
        include_str!("../../../../../../zircon_plugins/net/editor/src/plugin.rs"),
        include_str!(
            "../../../../../../zircon_plugins/net/editor/src/tests/authoring_extensions.rs"
        ),
        include_str!("../../../../../../zircon_plugins/net/editor/Cargo.toml"),
        "zircon_plugin_net_runtime::NET_RUNTIME_CAPABILITY",
    ),
];

#[test]
fn review_d9_editor_runtime_mirror_consumers_use_sdk_declaration() {
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
    );
    let capability_audit =
        include_str!("../../../../../../tools/plugin_structure_audits/capability.py");
    let audit_report = include_str!("../../../../../../tools/audit_plugin_structure.py");
    let catalog =
        include_str!("../../../../../../zircon_plugins/first_party_runtime_catalog/src/lib.rs");

    assert_eq!(
        D9_EDITOR_RUNTIME_MIRROR_CRATES.len(),
        3,
        "D9 editor/runtime mirror guard should cover animation, physics, and net representative roots"
    );

    for &(label, plugin_source, test_source, cargo_toml, runtime_capability) in
        D9_EDITOR_RUNTIME_MIRROR_CRATES
    {
        let runtime_crate = format!("zircon_plugin_{label}_runtime");
        let mirror_method =
            format!(".mirrors_runtime_manifest({runtime_crate}::package_manifest())");
        let mirror_macro = format!("mirrors_runtime_manifest: {runtime_crate}::package_manifest()");
        assert!(
            plugin_source.contains("EditorPluginDeclaration"),
            "{label} editor plugin should be declared through EditorPluginDeclaration"
        );
        assert!(
            plugin_source.contains(&mirror_method) || plugin_source.contains(&mirror_macro),
            "{label} editor declaration should explicitly mirror its runtime package manifest"
        );
        assert!(
            plugin_source.contains("plugin.declaration().registration_report(&plugin)"),
            "{label} registration should flow through EditorPluginDeclaration"
        );
        for stale in [
            "EditorPluginRegistrationReport::from_plugin(",
            "editor_plugin().declaration().registration_report(&editor_plugin())",
        ] {
            assert!(
                !plugin_source.contains(stale),
                "{label} editor plugin should not keep stale implicit registration path `{stale}`"
            );
        }
        assert!(
            cargo_toml
                .contains("zircon_plugin_sdk = { workspace = true, features = [\"editor\"] }"),
            "{label} editor crate should inherit the SDK editor dependency from the plugin workspace"
        );
        for required in ["mirrored_runtime_package_id()", ".package_manifest"] {
            assert!(
                test_source.contains(required),
                "{label} editor tests should assert mirrored runtime package evidence `{required}`"
            );
        }
        assert!(
            test_source.contains(runtime_capability),
            "{label} editor tests should assert mirrored runtime capability `{runtime_capability}`"
        );
    }

    for required in [
        "FIRST_PARTY_EDITOR_RUNTIME_MIRROR_ROOTS",
        "collect_editor_runtime_mirror_violations",
        "editor_runtime_mirror_violations",
        "d9_editor_runtime_mirror_gate_status",
        "editor-runtime-mirror-clean",
    ] {
        assert!(
            capability_audit.contains(required) || audit_report.contains(required),
            "plugin structure audit should expose D9 editor/runtime mirror anchor `{required}`"
        );
    }
    for required in [
        "\\\"editor_runtime_mirror_root_count\\\": 3",
        "\\\"editor_runtime_mirror_violations\\\": 0",
        "\\\"d9_editor_runtime_mirror_gate_status\\\": \\\"editor-runtime-mirror-clean\\\"",
    ] {
        assert!(
            catalog.contains(required),
            "first-party runtime catalog should lock D9 audit JSON anchor `{required}`"
        );
    }

    let d9_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D9 |"))
        .expect("D9 review finding row should exist");
    for required in [
        "editor/runtime mirror consumer guard",
        "EditorPluginDeclaration::mirrors_runtime_manifest",
        "d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred",
        "d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean",
    ] {
        assert!(
            review_findings.contains(required),
            "D9 numbered review evidence should record editor/runtime mirror consumer convergence anchor `{required}`"
        );
    }
    assert!(
        !d9_row.contains("无机器校验对齐"),
        "D9 row should not keep the stale unguarded-asymmetry wording"
    );

    assert!(
        review_findings.contains("D9 editor/runtime mirror consumer guard")
            && review_findings
                .contains("review_d9_editor_runtime_mirror_consumers_use_sdk_declaration"),
        "D9 numbered output should own the concrete editor/runtime mirror evidence"
    );
}

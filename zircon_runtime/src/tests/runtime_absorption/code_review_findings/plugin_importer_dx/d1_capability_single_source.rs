const D1_RUNTIME_CAPABILITY_ROOTS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "ai",
        include_str!("../../../../../../zircon_plugins/ai/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/ai/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/ai/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/ai/plugin.toml"),
    ),
    (
        "animation",
        include_str!("../../../../../../zircon_plugins/animation/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/animation/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/animation/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/animation/plugin.toml"),
    ),
    (
        "hybrid_gi",
        include_str!("../../../../../../zircon_plugins/hybrid_gi/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/hybrid_gi/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/hybrid_gi/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/hybrid_gi/plugin.toml"),
    ),
    (
        "navigation",
        include_str!("../../../../../../zircon_plugins/navigation/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/navigation/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/navigation/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/navigation/plugin.toml"),
    ),
    (
        "net",
        include_str!("../../../../../../zircon_plugins/net/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/net/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/net/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/net/plugin.toml"),
    ),
    (
        "particles",
        include_str!("../../../../../../zircon_plugins/particles/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/particles/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/particles/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/particles/plugin.toml"),
    ),
    (
        "physics",
        include_str!("../../../../../../zircon_plugins/physics/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/physics/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/physics/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/physics/plugin.toml"),
    ),
    (
        "prefab_tools",
        include_str!("../../../../../../zircon_plugins/prefab_tools/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/prefab_tools/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/prefab_tools/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/prefab_tools/plugin.toml"),
    ),
    (
        "rendering",
        include_str!("../../../../../../zircon_plugins/rendering/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/rendering/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/rendering/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/rendering/plugin.toml"),
    ),
    (
        "solari",
        include_str!("../../../../../../zircon_plugins/solari/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/solari/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/solari/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/solari/plugin.toml"),
    ),
    (
        "terrain",
        include_str!("../../../../../../zircon_plugins/terrain/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/terrain/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/terrain/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/terrain/plugin.toml"),
    ),
    (
        "texture",
        include_str!("../../../../../../zircon_plugins/texture/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/texture/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/texture/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/texture/plugin.toml"),
    ),
    (
        "tilemap_2d",
        include_str!("../../../../../../zircon_plugins/tilemap_2d/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/tilemap_2d/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/tilemap_2d/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/tilemap_2d/plugin.toml"),
    ),
    (
        "virtual_geometry",
        include_str!("../../../../../../zircon_plugins/virtual_geometry/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/virtual_geometry/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/virtual_geometry/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/virtual_geometry/plugin.toml"),
    ),
    (
        "zr_vm_language",
        include_str!("../../../../../../zircon_plugins/zr_vm_language/runtime/src/capability.rs"),
        include_str!("../../../../../../zircon_plugins/zr_vm_language/runtime/src/lib.rs"),
        include_str!("../../../../../../zircon_plugins/zr_vm_language/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/zr_vm_language/plugin.toml"),
    ),
];

#[test]
fn review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror() {
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let plugin_audit_doc =
        include_str!("../../../../../../docs/zircon_plugins/plugin_structure_audits.md");
    let runtime_15 = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let capability_audit =
        include_str!("../../../../../../tools/plugin_structure_audits/capability.py");
    let audit_cli = include_str!("../../../../../../tools/audit_plugin_structure.py");
    let catalog =
        include_str!("../../../../../../zircon_plugins/first_party_runtime_catalog/src/lib.rs");
    let feature_bundle_builder = include_str!(
        "../../../../../../zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs"
    );
    let manifest_mod =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/manifest/mod.rs");
    let plugin_sdk_lib = include_str!("../../../../../../zircon_plugins/plugin_sdk/src/lib.rs");
    let plugin_sdk_prelude =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/prelude.rs");
    let editor_sdk = include_str!("../../../../../../zircon_plugins/plugin_sdk/src/editor.rs");
    let manifest_tests =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/manifest/tests.rs");

    assert_eq!(
        D1_RUNTIME_CAPABILITY_ROOTS.len(),
        15,
        "D1 capability single-source guard should cover every first-party trait-backed runtime root"
    );
    for (root, capability_source, lib_source, plugin_source, manifest_source) in
        D1_RUNTIME_CAPABILITY_ROOTS
    {
        assert!(
            capability_source.contains("pub const RUNTIME_CAPABILITIES: &[&str]")
                && capability_source.contains("&["),
            "{root} runtime capability.rs should own the runtime capability slice"
        );
        assert!(
            capability_source
                .lines()
                .any(|line| line.trim_start().starts_with("pub const ")
                    && line.contains("CAPABILITY")
                    && line.contains(": &str")),
            "{root} runtime capability.rs should own named capability constants"
        );
        assert!(
            lib_source.contains("mod capability;"),
            "{root} runtime lib.rs should mount capability.rs as the single source"
        );
        assert!(
            lib_source.contains("pub use capability::")
                && lib_source.contains("RUNTIME_CAPABILITIES"),
            "{root} runtime lib.rs should re-export capability.rs constants instead of restating them"
        );
        assert!(
            lib_source.contains("runtime_capabilities"),
            "{root} runtime lib.rs should expose the runtime capability accessor"
        );
        for (line_number, line) in lib_source.lines().enumerate() {
            let trimmed = line.trim_start();
            assert!(
                !(trimmed.starts_with("pub const ")
                    && trimmed.contains("CAPABILITY")
                    && trimmed.contains(": &str")),
                "{root} runtime lib.rs should not redeclare capability constants at line {}",
                line_number + 1
            );
        }
        assert!(
            plugin_source.contains("pub fn runtime_capabilities() -> &'static [&'static str]")
                && plugin_source.contains("RUNTIME_CAPABILITIES"),
            "{root} runtime plugin.rs should project capability.rs through the SDK-facing accessor"
        );
        assert!(
            manifest_source.contains("capabilities = [") && manifest_source.contains("[[modules]]"),
            "{root} plugin.toml should keep root and module capability lists auditable"
        );
    }

    assert_contains_all(
        "plugin capability audit enforces capability.rs as the runtime root single source",
        capability_audit,
        &[
            "FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS",
            "audited_runtime_root_count",
            "capability_source_mismatches",
            "missing_capability_owner_files",
            "missing_runtime_capability_exports",
            "root_capability_mismatches",
            "module_capability_mismatches",
            "lib_capability_literal_sites",
            "m4_runtime_capability_gate_status",
            "runtime-capability-single-source-clean",
            "sdk_builder_mirror_violations",
            "m4_t2_builder_mirror_gate_status",
            "sdk-builder-mirror-clean",
            "PluginFeatureBundleBuilder",
        ],
    );
    assert_contains_all(
        "plugin structure audit CLI wires D1 capability audit into the JSON result",
        audit_cli,
        &[
            "from plugin_structure_audits.capability import audit_plugin_capability_conformance",
            "capability_conformance = audit_plugin_capability_conformance(root).to_json()",
            "\"capability_conformance\": capability_conformance",
            "\"capability_source_mismatches\": capability_conformance[",
            "\"m4_runtime_capability_gate_status\": capability_conformance[",
            "\"sdk_builder_mirror_violations\": capability_conformance[",
            "\"m4_t2_builder_mirror_gate_status\": capability_conformance[",
        ],
    );
    assert_contains_all(
        "first-party runtime catalog keeps the D1 capability audit as an executable guard",
        catalog,
        &[
            "plugins_12_capability_single_source_conformance",
            r#"\"audited_runtime_root_count\": 15"#,
            r#"\"capability_source_mismatches\": 0"#,
            r#"\"missing_capability_owner_files\": 0"#,
            r#"\"missing_runtime_capability_exports\": 0"#,
            r#"\"root_capability_mismatches\": 0"#,
            r#"\"module_capability_mismatches\": 0"#,
            r#"\"lib_capability_literal_sites\": 0"#,
            r#"\"m4_runtime_capability_gate_status\""#,
            "runtime-capability-single-source-clean",
            r#"\"sdk_builder_mirror_violations\": 0"#,
            r#"\"m4_t2_builder_mirror_gate_status\""#,
            "sdk-builder-mirror-clean",
        ],
    );
    assert_contains_all(
        "SDK feature bundle builder mirrors capability declarations into feature and module manifests",
        feature_bundle_builder,
        &[
            "pub struct PluginFeatureBundleBuilder",
            "pub fn with_runtime_capability_module",
            "pub fn with_editor_capability_module",
            "let capability = capability.into();",
            "PluginModuleManifest::runtime(module_name, crate_name)",
            ".with_target_modes(target_modes)",
            ".with_capabilities([capability.clone()])",
            "self.with_capability(capability).with_runtime_module(module)",
            "PluginModuleManifest::editor(module_name, crate_name)",
            "self.with_capability(capability).with_editor_module(module)",
        ],
    );
    assert_contains_all(
        "SDK builder and mirror APIs are exported through stable plugin SDK surfaces",
        &format!("{manifest_mod}\n{plugin_sdk_lib}\n{plugin_sdk_prelude}\n{editor_sdk}"),
        &[
            "pub use feature_bundle_builder::PluginFeatureBundleBuilder",
            "PluginFeatureBundleBuilder",
            "pub fn mirrors_runtime(",
            "pub fn mirrors_runtime_manifest(",
            "pub fn mirrored_runtime_package_id(",
            "mirrors_runtime: $runtime_declaration:expr",
            "editor_declaration_mirrors_runtime_manifest_and_keeps_editor_capabilities",
        ],
    );
    assert_contains_all(
        "SDK manifest tests lock the builder capability projection contract",
        manifest_tests,
        &[
            "feature_bundle_builder_projects_capability_to_feature_and_modules",
            "with_runtime_capability_module",
            "with_editor_capability_module",
            "feature.capabilities",
            "feature.modules[0].capabilities",
            "feature.modules[1].capabilities",
        ],
    );

    let d1_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D1 |"))
        .expect("engine code review findings should keep a D1 row");
    assert_contains_all(
        "D1 review finding row is closed by capability single-source and SDK builder evidence",
        d1_row,
        &[
            "已闭合",
            "15 个 trait-backed first-party runtime roots",
            "plugins_12_runtime_capability_single_source_guard_passed",
            "plugins_12_capability_single_source_conformance",
            "m4_runtime_capability_gate_status = runtime-capability-single-source-clean",
            "capability_source_mismatches = 0",
            "m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean",
            "sdk_builder_mirror_violations = 0",
            "review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror",
            "d1_capability_single_source_review_synced_static_passed_cargo_deferred",
        ],
    );
    assert!(
        !d1_row.contains("改一个名动 6 处") && !d1_row.contains("重复 3 次"),
        "D1 row should no longer describe capability duplication as an open issue"
    );

    for (doc_name, doc) in [
        ("engine-code-review-findings", review_findings),
        ("engine-code-structure-convention", structure_convention),
        ("plugins 12 plan", plugins_12),
        ("plugin SDK docs", plugin_sdk_doc),
        ("plugin structure audit docs", plugin_audit_doc),
        ("runtime 15 plan", runtime_15),
        ("runtime index", runtime_index),
        ("runtime module convention docs", module_convention),
        ("coordination session note", session_note),
    ] {
        assert_contains_all(
            doc_name,
            doc,
            &[
                "D1 capability single-source review/status sync",
                "d1_capability_single_source_review_synced_static_passed_cargo_deferred",
                "review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror",
                "plugins_12_runtime_capability_single_source_guard_passed",
                "plugins_12_capability_single_source_conformance",
                "m4_runtime_capability_gate_status = runtime-capability-single-source-clean",
                "capability_source_mismatches = 0",
                "m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean",
                "sdk_builder_mirror_violations = 0",
            ],
        );
    }
}

fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    for needle in needles {
        assert!(source.contains(needle), "{label} should contain `{needle}`");
    }
}

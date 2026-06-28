#[path = "plugin_importer_dx/d10_bridge_call.rs"]
mod d10_bridge_call;
// D10 child anchors: fn review_d10_animation_physics_tests_use_sdk_bridge_call,
// d10_animation_physics_bridge_call_static_passed_cargo_deferred,
// WeakBridge<dyn PhysicsQueryInterface>.
#[path = "plugin_importer_dx/d11_test_runtime_fixture.rs"]
mod d11_test_runtime_fixture;
#[path = "plugin_importer_dx/d12_runtime_exports.rs"]
mod d12_runtime_exports;
#[path = "plugin_importer_dx/d13_importer_sdk.rs"]
mod d13_importer_sdk;
#[path = "plugin_importer_dx/d1_capability_single_source.rs"]
mod d1_capability_single_source;
#[path = "plugin_importer_dx/d6_runtime_plugin_id.rs"]
mod d6_runtime_plugin_id;
// D6 child anchors: fn review_d6_runtime_plugin_id_accepts_external_string_keys,
// d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred.

const D8_RUNTIME_REGISTRATION_CRATES: &[(&str, &str, &str)] = &[
    (
        "animation",
        include_str!("../../../../../zircon_plugins/animation/runtime/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/animation/runtime/src/runtime_system.rs"),
    ),
    (
        "physics",
        include_str!("../../../../../zircon_plugins/physics/runtime/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/physics/runtime/src/runtime_system.rs"),
    ),
    (
        "net",
        include_str!("../../../../../zircon_plugins/net/runtime/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/net/runtime/src/runtime_system.rs"),
    ),
];

const D9_EDITOR_RUNTIME_MIRROR_CRATES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "animation",
        include_str!("../../../../../zircon_plugins/animation/editor/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/animation/editor/src/tests.rs"),
        include_str!("../../../../../zircon_plugins/animation/editor/Cargo.toml"),
        "zircon_plugin_animation_runtime::ANIMATION_RUNTIME_CAPABILITY",
    ),
    (
        "physics",
        include_str!("../../../../../zircon_plugins/physics/editor/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/physics/editor/src/tests.rs"),
        include_str!("../../../../../zircon_plugins/physics/editor/Cargo.toml"),
        "zircon_plugin_physics_runtime::PHYSICS_RUNTIME_CAPABILITY",
    ),
    (
        "net",
        include_str!("../../../../../zircon_plugins/net/editor/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/net/editor/src/tests/authoring_extensions.rs"),
        include_str!("../../../../../zircon_plugins/net/editor/Cargo.toml"),
        "zircon_plugin_net_runtime::NET_RUNTIME_CAPABILITY",
    ),
];

const D5_EDITOR_AUTHORING_MACRO_CRATES: &[(&str, &str, &str)] = &[
    (
        "animation",
        include_str!("../../../../../zircon_plugins/animation/editor/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/animation/editor/src/tests.rs"),
    ),
    (
        "physics",
        include_str!("../../../../../zircon_plugins/physics/editor/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/physics/editor/src/tests.rs"),
    ),
    (
        "net",
        include_str!("../../../../../zircon_plugins/net/editor/src/plugin.rs"),
        include_str!("../../../../../zircon_plugins/net/editor/src/tests/authoring_extensions.rs"),
    ),
];

#[test]
fn review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder() {
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../docs/zircon_plugins/plugin-sdk.md");
    let plugin_skeleton_doc =
        include_str!("../../../../../docs/zircon_plugins/plugin-crate-skeleton.md");
    let physics_doc = include_str!("../../../../../docs/zircon_plugins/physics/runtime.md");
    let net_doc = include_str!("../../../../../docs/zircon_plugins/net/runtime.md");
    let runtime_15 = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_convention =
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let sdk_registration =
        include_str!("../../../../../zircon_plugins/plugin_sdk/src/registration.rs");
    let registration_audit =
        include_str!("../../../../../tools/plugin_structure_audits/registration.py");
    let audit_report = include_str!("../../../../../tools/audit_plugin_structure.py");

    assert_eq!(
        D8_RUNTIME_REGISTRATION_CRATES.len(),
        3,
        "D8 registration builder guard should cover animation, physics and net original evidence roots"
    );

    for (label, plugin_source, runtime_system_source) in D8_RUNTIME_REGISTRATION_CRATES {
        for required in [
            "RuntimePluginRegistrationBuilder::new(registry)",
            ".module(PLUGIN_RUNTIME_MODULE_NAME, module_descriptor())",
        ] {
            assert!(
                plugin_source.contains(required),
                "{label} plugin owner should route module registration through SDK builder `{required}`"
            );
        }
        for stale in ["intern_plugin_module(", "register_module("] {
            assert!(
                !plugin_source.contains(stale),
                "{label} plugin owner should not hand-write registry call `{stale}`"
            );
        }

        for required in ["RuntimePluginModuleRegistration", ".runtime_scene_system("] {
            assert!(
                runtime_system_source.contains(required),
                "{label} runtime system owner should declare systems through SDK module handle `{required}`"
            );
        }
        for stale in [
            "PluginModuleId",
            "RuntimeExtensionRegistry,",
            "register_runtime_scene_system(",
            "intern_system_set(",
            "register_event::<",
        ] {
            assert!(
                !runtime_system_source.contains(stale),
                "{label} runtime system owner should not reach for low-level registry API `{stale}`"
            );
        }
    }

    for required in [
        "pub fn event<E>",
        "PluginEventManifest",
        "register_event::<E>",
        "pub fn plugin_option",
        "pub fn plugin_event_catalog",
        "register_runtime_scene_system(self.owner",
    ] {
        assert!(
            sdk_registration.contains(required),
            "plugin SDK registration builder should own runtime registration helper `{required}`"
        );
    }

    for required in [
        "D8_RUNTIME_REGISTRATION_ROOTS",
        "runtime_registration_builder_violation_count",
        "m3_t2_runtime_registration_builder_status",
        "runtime-registration-builder-clean",
    ] {
        assert!(
            registration_audit.contains(required) || audit_report.contains(required),
            "plugin structure audit should expose D8 runtime registration builder anchor `{required}`"
        );
    }

    let d8_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D8 |"))
        .expect("D8 review finding row should exist");
    for required in [
        "animation/physics/net 原始证据路径已收敛",
        "RuntimePluginRegistrationBuilder",
        "RuntimePluginModuleRegistration::event",
        "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred",
        "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
    ] {
        assert!(
            d8_row.contains(required),
            "D8 row should record runtime registration builder convergence anchor `{required}`"
        );
    }
    assert!(
        !d8_row.contains("三步样板每插件手抄"),
        "D8 row should not keep the stale open-problem wording"
    );

    for (doc_label, doc) in [
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("Plugins 12 plan", plugins_12),
        ("plugin SDK doc", plugin_sdk_doc),
        ("plugin skeleton doc", plugin_skeleton_doc),
        ("physics plugin doc", physics_doc),
        ("net plugin doc", net_doc),
        ("Runtime 15 plan", runtime_15),
        ("Runtime index", runtime_index),
        ("module convention", module_convention),
        ("session note", session_note),
    ] {
        for required in [
            "D8 runtime registration builder original evidence paths",
            "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred",
            "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
            "RuntimePluginRegistrationBuilder",
        ] {
            assert!(
                doc.contains(required),
                "{doc_label} should record D8 runtime registration builder convergence anchor `{required}`"
            );
        }
    }
}

#[test]
fn review_d5_editor_authoring_plugins_use_sdk_macro() {
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../docs/zircon_plugins/plugin-sdk.md");
    let runtime_15 = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_convention =
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let sdk_editor = include_str!("../../../../../zircon_plugins/plugin_sdk/src/editor.rs");

    assert_eq!(
        D5_EDITOR_AUTHORING_MACRO_CRATES.len(),
        3,
        "D5 editor authoring macro guard should cover animation, physics, and net representative roots"
    );

    for &(label, plugin_source, test_source) in D5_EDITOR_AUTHORING_MACRO_CRATES {
        for required in [
            "authoring_plugin! {",
            "mirrors_runtime_manifest:",
            "capabilities: EDITOR_CAPABILITIES",
            "register_extensions:",
            "pub fn editor_plugin_declaration() -> EditorPluginDeclaration",
            "plugin.declaration().registration_report(&plugin)",
        ] {
            assert!(
                plugin_source.contains(required),
                "{label} editor plugin should use SDK authoring macro anchor `{required}`"
            );
        }
        for stale in [
            "impl zircon_editor::EditorPlugin for",
            "pub struct AnimationEditorPlugin {\n    declaration: EditorPluginDeclaration",
            "pub struct PhysicsEditorPlugin {\n    declaration: EditorPluginDeclaration",
            "pub struct NetEditorPlugin {\n    declaration: EditorPluginDeclaration",
            "EditorPluginDeclaration::new(",
        ] {
            assert!(
                !plugin_source.contains(stale),
                "{label} editor plugin should not keep hand-written editor boilerplate `{stale}`"
            );
        }
        for required in [
            "mirrored_runtime_package_id()",
            ".package_manifest",
            "_AUTHORING_CAPABILITY",
        ] {
            assert!(
                test_source.contains(required),
                "{label} editor tests should keep macro consumer evidence `{required}`"
            );
        }
    }

    for required in [
        "macro_rules! authoring_plugin",
        "mirrors_runtime_manifest: $runtime_manifest:expr",
        "let declaration = declaration.mirrors_runtime_manifest($runtime_manifest);",
        "pub fn declaration(&self) -> &$crate::editor::EditorPluginDeclaration",
        "pub fn registration_report",
    ] {
        assert!(
            sdk_editor.contains(required),
            "plugin SDK editor macro should own D5 helper anchor `{required}`"
        );
    }

    let d5_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D5 |"))
        .expect("D5 review finding row should exist");
    for required in [
        "editor authoring macro consumer guard",
        "animation/physics/net",
        "zircon_plugin_sdk::authoring_plugin!",
        "d5_editor_authoring_macro_consumers_static_passed_cargo_deferred",
        "review_d5_editor_authoring_plugins_use_sdk_macro",
    ] {
        assert!(
            d5_row.contains(required),
            "D5 row should record editor authoring macro consumer convergence anchor `{required}`"
        );
    }
    assert!(
        !d5_row.contains("editor 插件逐字节复制模板"),
        "D5 row should not keep the stale copy-paste template wording as current state"
    );

    for (doc_label, doc) in [
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("Plugins 12 plan", plugins_12),
        ("plugin SDK doc", plugin_sdk_doc),
        ("Runtime 15 plan", runtime_15),
        ("Runtime index", runtime_index),
        ("module convention", module_convention),
        ("session note", session_note),
    ] {
        for required in [
            "D5 editor authoring macro consumer guard",
            "d5_editor_authoring_macro_consumers_static_passed_cargo_deferred",
            "review_d5_editor_authoring_plugins_use_sdk_macro",
            "zircon_plugin_sdk::authoring_plugin!",
        ] {
            assert!(
                doc.contains(required),
                "{doc_label} should record D5 editor authoring macro consumer anchor `{required}`"
            );
        }
    }
}

#[test]
fn review_d9_editor_runtime_mirror_consumers_use_sdk_declaration() {
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../docs/zircon_plugins/plugin-sdk.md");
    let plugin_audit_doc =
        include_str!("../../../../../docs/zircon_plugins/plugin_structure_audits.md");
    let animation_doc = include_str!("../../../../../docs/zircon_plugins/animation/runtime.md");
    let physics_doc = include_str!("../../../../../docs/zircon_plugins/physics/runtime.md");
    let net_doc = include_str!("../../../../../docs/zircon_plugins/net/runtime.md");
    let runtime_15 = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_convention =
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let capability_audit =
        include_str!("../../../../../tools/plugin_structure_audits/capability.py");
    let audit_report = include_str!("../../../../../tools/audit_plugin_structure.py");
    let catalog =
        include_str!("../../../../../zircon_plugins/first_party_runtime_catalog/src/lib.rs");

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
            d9_row.contains(required),
            "D9 row should record editor/runtime mirror consumer convergence anchor `{required}`"
        );
    }
    assert!(
        !d9_row.contains("无机器校验对齐"),
        "D9 row should not keep the stale unguarded-asymmetry wording"
    );

    for (doc_label, doc) in [
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("Plugins 12 plan", plugins_12),
        ("plugin SDK doc", plugin_sdk_doc),
        ("plugin audit doc", plugin_audit_doc),
        ("animation plugin doc", animation_doc),
        ("physics plugin doc", physics_doc),
        ("net plugin doc", net_doc),
        ("Runtime 15 plan", runtime_15),
        ("Runtime index", runtime_index),
        ("module convention", module_convention),
        ("session note", session_note),
    ] {
        for required in [
            "D9 editor/runtime mirror consumer guard",
            "d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred",
            "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
            "d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean",
        ] {
            assert!(
                doc.contains(required),
                "{doc_label} should record D9 editor/runtime mirror consumer anchor `{required}`"
            );
        }
    }
}

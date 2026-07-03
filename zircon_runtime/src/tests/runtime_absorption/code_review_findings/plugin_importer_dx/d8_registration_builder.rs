const D8_RUNTIME_REGISTRATION_CRATES: &[(&str, &str, &str)] = &[
    (
        "animation",
        include_str!("../../../../../../zircon_plugins/animation/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/animation/runtime/src/runtime_system.rs"),
    ),
    (
        "physics",
        include_str!("../../../../../../zircon_plugins/physics/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/physics/runtime/src/runtime_system.rs"),
    ),
    (
        "net",
        include_str!("../../../../../../zircon_plugins/net/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/net/runtime/src/runtime_system.rs"),
    ),
];

#[test]
fn review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder() {
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let plugin_skeleton_doc =
        include_str!("../../../../../../docs/zircon_plugins/plugin-crate-skeleton.md");
    let physics_doc = include_str!("../../../../../../docs/zircon_plugins/physics/runtime.md");
    let net_doc = include_str!("../../../../../../docs/zircon_plugins/net/runtime.md");
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
    let sdk_registration =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/registration.rs");
    let registration_audit =
        include_str!("../../../../../../tools/plugin_structure_audits/registration.py");
    let audit_report = include_str!("../../../../../../tools/audit_plugin_structure.py");

    assert_eq!(
        D8_RUNTIME_REGISTRATION_CRATES.len(),
        3,
        "D8 registration builder guard should cover animation, physics and net original evidence roots"
    );

    for (label, plugin_source, runtime_system_source) in D8_RUNTIME_REGISTRATION_CRATES {
        assert!(
            plugin_source.contains("RuntimePluginRegistrationBuilder::new(registry)")
                && plugin_source.contains(".module(")
                && plugin_source.contains("PLUGIN_RUNTIME_MODULE_NAME")
                && (plugin_source.contains("module_descriptor())")
                    || plugin_source.contains("module_descriptor_with_manager(")),
            "{label} plugin owner should route module registration through SDK builder module helpers"
        );
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

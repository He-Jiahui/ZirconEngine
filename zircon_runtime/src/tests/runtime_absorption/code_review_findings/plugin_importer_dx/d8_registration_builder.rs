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
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
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
            review_findings.contains(required),
            "D8 numbered review evidence should record runtime registration builder convergence anchor `{required}`"
        );
    }
    assert!(
        !d8_row.contains("三步样板每插件手抄"),
        "D8 row should not keep the stale open-problem wording"
    );

    assert!(
        review_findings.contains("D8 runtime registration builder original evidence paths")
            && review_findings.contains(
                "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
            ),
        "D8 numbered output should own the concrete registration-builder evidence"
    );
}

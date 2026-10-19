//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
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

#[test]
fn review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder() {
    let review_findings = crate::runtime_test_history_concat!(
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../docs/architecture/runtime-architecture-review-m0.md")
    );
    let sdk_registration =
        include_str!("../../../../../zircon_plugins/plugin_sdk/src/registration.rs");
    let registration_audit = include_str!("../../../../../tools/audits/plugins/registration.py");
    let audit_report = include_str!("../../../../../tools/audits/audit_plugin_structure.py");

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

        let normalized = runtime_system_source.lines().collect::<Vec<_>>().join("\n");
        let native = normalized
            .split_once("pub fn register_runtime_system")
            .expect("each original owner must mount its native registration function")
            .1
            .split_once("\n}\n")
            .expect("native registration function must have its complete body")
            .0;
        for required in ["RuntimePluginModuleRegistration", ".runtime_scene_system("] {
            assert!(
                native.contains(required),
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
                !native.contains(stale),
                "{label} runtime system owner should not reach for low-level registry API `{stale}`"
            );
        }
    }

    let animation_system =
        include_str!("../../../../../zircon_plugins/animation/runtime/src/runtime_system.rs");
    let normalized = animation_system.lines().collect::<Vec<_>>().join("\n");
    assert!(
        normalized.contains("#[cfg(feature = \"native-host\")]\npub fn register_runtime_system(")
    );
    let portable = normalized
        .split_once("pub fn portable_runtime_extension_plan()")
        .expect("portable animation registration must retain its independent World plan")
        .1;
    for required in [
        "WorldRuntimeExtensionPlan::from_registrations([WorldRuntimeExtensionRegistration::new(",
        "world.register_event::<crate::AnimationClipEvent>();",
        "world.register_event::<crate::AnimationEvaluationDiagnostic>();",
        "world.register_event::<crate::AnimationStateMachineLayerDiagnostic>();",
        "world.insert_resource(crate::AnimationEvaluationPipeline::default());",
        "register_boxed_runtime_scene_system",
        "SceneSystemMetadata::new(ANIMATION_EVALUATE_SYSTEM, SystemStage::PostUpdate, 0)",
        "run_animation_runtime_system",
        "WorldRuntimeExtensionError::registration_failed(",
    ] {
        assert!(
            portable.contains(required),
            "portable World plan missing {required}"
        );
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

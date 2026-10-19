//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
#[test]
fn review_f5_fixed_world_mutation_uses_scene_error_variants() {
    let world_error = include_str!("../../../../../scene/world/error.rs");
    let component_access = include_str!("../../../../../scene/world/component_access.rs");
    let hierarchy = include_str!("../../../../../scene/world/hierarchy.rs");
    let query = include_str!("../../../../../scene/world/query.rs");
    let records = include_str!("../../../../../scene/world/records.rs");
    let review_findings = crate::runtime_test_history_concat!(
        include_str!("../../../../../../../docs/architecture/runtime-architecture-review-m0.md"),
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/crates/zircon_runtime/structure/module-convention.md"
    );
    let runtime_index = include_str!(
        "../../../../../../../docs/crates/zircon_runtime/structure/runtime-absorption-evidence.md"
    );
    let convention =
        include_str!("../../../../../../../docs/testing-and-validation/development-conventions.md");
    let ecs_doc = include_str!("../../../../../../../docs/crates/zircon_runtime/scene/ecs.md");

    for required in [
        "MissingRequiredComponent",
        "DuplicateEntity",
        "EmptyNodeName",
        "JointConnectsToSelf",
        "EntityCannotParentItself",
        "MissingParent",
        "HierarchyCycle",
        "DynamicMobilityWithStaticChildren",
        "StaticMobilityUnderDynamicParent",
        "StaticTransformMutation",
        "StaticReparentMutation",
    ] {
        assert!(
            world_error.contains(required),
            "F5 fixed world mutation SceneError should expose `{required}`"
        );
    }

    for (label, source) in [
        ("component access", component_access),
        ("hierarchy", hierarchy),
        ("query", query),
        ("records", records),
    ] {
        for forbidden in [
            "Err(format!(",
            "to_string().into()",
            "Result<(), String>",
            "Result<bool, String>",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep fixed world mutation String error branch `{forbidden}`"
            );
        }
    }

    for required in [
        "SceneError::missing_entity(\"update rigid body for\", entity)",
        "SceneError::JointConnectsToSelf",
        "SceneError::missing_entity(\"reparent\", child)",
        "SceneError::EntityCannotParentItself",
        "SceneError::MissingParent",
        "SceneError::HierarchyCycle",
        "SceneError::DynamicMobilityWithStaticChildren",
        "SceneError::StaticMobilityUnderDynamicParent",
        "SceneError::StaticTransformMutation",
        "SceneError::StaticReparentMutation",
        "SceneError::MissingRequiredComponent",
        "\"update active state for\"",
        "SceneError::missing_entity(\"update mobility for\", entity)",
        "SceneError::DuplicateEntity",
        "SceneError::EmptyNodeName",
        "SceneError::missing_entity(\"rename\", entity)",
    ] {
        assert!(
            component_access.contains(required)
                || hierarchy.contains(required)
                || query.contains(required)
                || records.contains(required),
            "fixed world mutation owners should contain `{required}`"
        );
    }

    for doc_anchor in [
        "F5 fixed world mutation typed errors",
        "runtime_15_fixed_world_mutation_typed_errors_static_passed_cargo_deferred",
        "review_f5_fixed_world_mutation_uses_scene_error_variants",
        "SceneError::MissingRequiredComponent",
        "scene/world/component_access.rs",
        "scene/world/hierarchy.rs",
        "scene/world/query.rs",
        "scene/world/records.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || ecs_doc.contains(doc_anchor),
            "F5 fixed world mutation docs should record `{doc_anchor}`"
        );
    }
}

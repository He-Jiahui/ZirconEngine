#[test]
fn review_f5_animation_manager_uses_animation_error() {
    let animation_mod = include_str!("../../../../core/framework/animation/mod.rs");
    let animation_error = include_str!("../../../../core/framework/animation/error.rs");
    let framework_manager = include_str!("../../../../core/framework/animation/manager.rs");
    let runtime_manager = include_str!("../../../../animation/manager/mod.rs");
    let pose = include_str!("../../../../animation/manager/pose.rs");
    let sampling = include_str!("../../../../animation/manager/sampling.rs");
    let sequence_compiled = include_str!("../../../../animation/sequence/compiled.rs");
    let sequence_conversion = include_str!("../../../../animation/sequence/conversion.rs");
    let review_findings = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );
    let convention = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
    );
    let animation_doc = include_str!("../../../../../../docs/zircon_runtime/animation/runtime.md");
    let framework_doc =
        include_str!("../../../../../../docs/zircon_runtime/core/framework/animation.md");

    for required in [
        "pub type AnimationResult<T> = std::result::Result<T, AnimationError>;",
        "pub enum AnimationError",
        "NonFiniteSkeletonBind",
        "ZeroLengthSkeletonBindRotation",
        "SampleTypeMismatch",
        "NonFiniteSample",
        "ZeroLengthQuaternionSample",
        "NonFiniteChannelSample",
        "ZeroLengthQuaternionChannelSample",
    ] {
        assert!(
            animation_error.contains(required),
            "F5 animation error owner should expose typed error anchor `{required}`"
        );
    }
    assert!(
        animation_mod.contains("pub use error::{AnimationError, AnimationResult};"),
        "AnimationError/AnimationResult should be exported through the animation framework facade"
    );
    for (label, source) in [
        ("framework animation manager", framework_manager),
        ("runtime animation manager", runtime_manager),
        ("animation pose sampler", pose),
        ("animation channel sampler", sampling),
        ("animation compiled sequence", sequence_compiled),
        ("animation sequence conversion", sequence_conversion),
    ] {
        for forbidden in [
            "Result<AnimationPoseOutput, String>",
            "Result<AnimationSequenceApplyReport, String>",
            "Result<AnimationPoseBone, String>",
            "Result<Vec3, String>",
            "Result<Quat, String>",
            "Result<ScenePropertyValue, String>",
            "Err(format!(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String animation error branch `{forbidden}`"
            );
        }
    }
    for required in [
        ") -> AnimationResult<AnimationPoseOutput>",
        ") -> AnimationResult<AnimationSequenceApplyReport>",
        "AnimationError::NonFiniteSkeletonBind",
        "AnimationError::ZeroLengthQuaternionChannelSample",
    ] {
        assert!(
            framework_manager.contains(required)
                || runtime_manager.contains(required)
                || pose.contains(required)
                || sampling.contains(required)
                || sequence_compiled.contains(required)
                || sequence_conversion.contains(required),
            "animation typed-error owners should contain `{required}`"
        );
    }
    for doc_anchor in [
        "F5 animation manager typed errors",
        "runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred",
        "review_f5_animation_manager_uses_animation_error",
        "AnimationError",
        "AnimationResult",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || animation_doc.contains(doc_anchor)
                || framework_doc.contains(doc_anchor),
            "F5 animation manager docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f6_core_resource_registry_rename_uses_resource_error() {
    let registry = include_str!("../../../../core/resource/registry.rs");
    let registry_ops = include_str!("../../../../core/resource/manager/registry_ops.rs");
    let resource_error = include_str!("../../../../core/resource/error.rs");
    let resource_mod = include_str!("../../../../core/resource/mod.rs");
    let runtime_error = include_str!("../../../../core/runtime/error.rs");
    let core_mod = include_str!("../../../../core/mod.rs");
    let resource_tests = include_str!("../../../../core/resource/tests.rs");
    let review_findings = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let runtime_02_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/02/2026-07-09-core-spine-and-root-surface-output-records.md"
    );
    let runtime_index = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );
    let convention = include_str!(
        "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
    );
    let resource_doc = include_str!("../../../../../../docs/zircon_runtime/core/resource.md");

    for required in [
        "pub type ResourceResult<T> = std::result::Result<T, ResourceRegistryError>;",
        "MissingRecordForLocator { locator: String }",
        "MissingRecordForId { id: String }",
    ] {
        assert!(
            resource_error.contains(required),
            "F6 ResourceRegistryError contract should contain `{required}`"
        );
    }
    assert!(
        resource_mod.contains("pub use error::{ResourceRegistryError, ResourceResult};"),
        "resource result should be exported only from the resource owner"
    );
    for forbidden in [
        "MissingResourceRecordForLocator",
        "MissingResourceRecordForId",
    ] {
        assert!(
            !runtime_error.contains(forbidden),
            "runtime CoreError must not recover legacy resource variant `{forbidden}`"
        );
    }
    for forbidden in ["ResourceRegistryError", "ResourceResult"] {
        assert!(
            !core_mod.contains(forbidden),
            "core root must not re-export or alias resource-owned `{forbidden}`"
        );
    }
    assert!(
        !runtime_error.contains("ResourceRegistryError")
            && !resource_error.contains("CoreError")
            && !registry.contains("CoreError")
            && !registry_ops.contains("CoreError"),
        "resource/framework errors must not be reconnected by a conversion shim"
    );
    for forbidden in [
        ") -> Result<ResourceRecord, String>",
        "Err(format!(\"missing resource record",
    ] {
        assert!(
            !registry.contains(forbidden) && !registry_ops.contains(forbidden),
            "F6 should not keep resource registry String error surface `{forbidden}`"
        );
    }
    for required in [
        ") -> ResourceResult<ResourceRecord>",
        "ResourceRegistryError::MissingRecordForLocator",
        "ResourceRegistryError::MissingRecordForId",
        "registry_rename_reports_missing_locator_with_resource_error",
    ] {
        assert!(
            registry.contains(required)
                || registry_ops.contains(required)
                || resource_tests.contains(required),
            "F6 resource registry rename should contain `{required}`"
        );
    }
    for doc_anchor in [
        "F6 core resource registry typed errors",
        "core_resource_registry_typed_errors_coremin_check_passed",
        "review_f6_core_resource_registry_rename_uses_resource_error",
        "MissingRecordForLocator",
        "f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_02_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || resource_doc.contains(doc_anchor),
            "F6 docs should record `{doc_anchor}`"
        );
    }
    let f6_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F6 |"))
        .expect("F6 review findings top row");
    assert!(
        f6_row.contains("f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred")
            && f6_row.ends_with("| Runtime 02 / review closed |"),
        "F6 top row should record typed-error review closed status"
    );
}

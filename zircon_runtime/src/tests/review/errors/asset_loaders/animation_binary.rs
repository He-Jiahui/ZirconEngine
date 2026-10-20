//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f5_animation_asset_binary_uses_typed_errors() {
    let animation_mod = include_str!("../../../../core/framework/animation/asset/mod.rs");
    let animation_error = include_str!("../../../../core/framework/animation/asset/error.rs");
    let binary = include_str!("../../../../core/framework/animation/asset/binary.rs");
    let channel = include_str!("../../../../core/framework/animation/asset/channel.rs");
    let clip = include_str!("../../../../core/framework/animation/asset/clip.rs");
    let graph = include_str!("../../../../core/framework/animation/asset/graph.rs");
    let reference = include_str!("../../../../core/framework/animation/asset/reference.rs");
    let sequence = include_str!("../../../../core/framework/animation/asset/sequence.rs");
    let skeleton = include_str!("../../../../core/framework/animation/asset/skeleton.rs");
    let state_machine = include_str!("../../../../core/framework/animation/asset/state_machine.rs");
    let importer_error = include_str!("../../../../asset/importer/error.rs");
    let import_animation_asset =
        include_str!("../../../../asset/importer/ingest/import_animation_asset.rs");
    let animation_tests = include_str!("../../../../asset/tests/assets/animation.rs");
    let review_findings = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let runtime_15_plan = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md");
    let runtime_index = history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let convention = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let animation_doc = include_str!(
        "../../../../../../docs/crates/zircon_runtime/core/framework/animation-assets.md"
    );

    for required in [
        "pub type AnimationAssetResult<T> = std::result::Result<T, AnimationAssetError>;",
        "pub enum AnimationAssetError",
        "Serialize {",
        "DocumentDeserialize {",
        "DocumentAndStreamDecode",
        "CurrentAndV1PayloadDecode",
        "InvalidReferenceUuid",
        "InvalidReferenceLocator",
        "KindMismatch",
        "UnknownGraphNodeTag",
    ] {
        assert!(
            animation_error.contains(required),
            "animation asset typed error owner should contain `{required}`"
        );
    }
    assert!(
        animation_mod.contains("pub use error::{AnimationAssetError, AnimationAssetResult, AnimationSkeletonHierarchyError};"),
        "animation module should export AnimationAssetError/AnimationAssetResult"
    );

    for required in [
        "pub enum AnimationSkeletonHierarchyError",
        "ParentCycle { bone_index: usize }",
        "ParentOutOfBounds {",
        "bone_count: usize",
        "InvalidSkeletonHierarchy {\n        #[source]\n        reason: AnimationSkeletonHierarchyError,",
    ] {
        assert!(animation_error.contains(required), "current core/framework/animation/asset/error.rs must retain {required}");
    }
    for required in ["AnimationSkeletonHierarchyError"] {
        assert!(
            animation_mod.contains(required),
            "current core/framework/animation/asset/mod.rs must retain {required}"
        );
    }
    for required in [
        "fn validate_hierarchy(&self) -> Result<(), AnimationSkeletonHierarchyError>",
        "AnimationSkeletonHierarchyError::ParentCycle",
        "AnimationSkeletonHierarchyError::ParentOutOfBounds",
        ".map_err(|reason| AnimationAssetError::InvalidSkeletonHierarchy { reason })?;",
        ".map_err(serde::de::Error::custom)?;",
    ] {
        assert!(
            skeleton.contains(required),
            "current core/framework/animation/asset/skeleton.rs must retain {required}"
        );
    }
    for (label, source) in [
        ("animation binary", binary),
        ("animation channel", channel),
        ("animation clip", clip),
        ("animation graph", graph),
        ("animation reference", reference),
        ("animation sequence", sequence),
        ("animation skeleton", skeleton),
        ("animation state machine", state_machine),
    ] {
        for forbidden in [
            "Result<Self, String>",
            "Result<Vec<u8>, String>",
            "type Error = String",
            "Err(format!(",
            ".map_err(|error| error.to_string())",
        ] {
            assert!(
                !production_section(source).contains(forbidden),
                "{label} should not keep lossy animation asset error branch `{forbidden}`"
            );
        }
    }

    for required in [
        "AnimationAsset(#[from] AnimationAssetError)",
        ".map_err(AssetImportError::AnimationAsset)",
        "AnimationAssetError::DocumentAndStreamDecode",
        "AnimationAssetError::CurrentAndV1PayloadDecode",
    ] {
        assert!(
            importer_error.contains(required)
                || import_animation_asset.contains(required)
                || animation_tests.contains(required),
            "animation import/test surface should contain `{required}`"
        );
    }

    for doc_anchor in [
        "F5 animation asset binary typed errors",
        "runtime_15_animation_asset_binary_typed_errors_static_passed_cargo_deferred",
        "review_f5_animation_asset_binary_uses_typed_errors",
        "AnimationAssetError::KindMismatch",
        "core/framework/animation/asset/error.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || animation_doc.contains(doc_anchor),
            "F5 animation asset docs should record `{doc_anchor}`"
        );
    }
}

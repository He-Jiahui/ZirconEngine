#[test]
fn review_f5_animation_asset_binary_uses_typed_errors() {
    let animation_mod = include_str!("../../../../../asset/assets/animation/mod.rs");
    let animation_error = include_str!("../../../../../asset/assets/animation/error.rs");
    let binary = include_str!("../../../../../asset/assets/animation/binary.rs");
    let channel = include_str!("../../../../../asset/assets/animation/channel.rs");
    let clip = include_str!("../../../../../asset/assets/animation/clip.rs");
    let graph = include_str!("../../../../../asset/assets/animation/graph.rs");
    let reference = include_str!("../../../../../asset/assets/animation/reference.rs");
    let sequence = include_str!("../../../../../asset/assets/animation/sequence.rs");
    let skeleton = include_str!("../../../../../asset/assets/animation/skeleton.rs");
    let state_machine = include_str!("../../../../../asset/assets/animation/state_machine.rs");
    let importer_error = include_str!("../../../../../asset/importer/error.rs");
    let import_animation_asset =
        include_str!("../../../../../asset/importer/ingest/import_animation_asset.rs");
    let animation_tests = include_str!("../../../../../asset/tests/assets/animation.rs");
    let review_findings = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );
    let convention = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
    );
    let animation_doc =
        include_str!("../../../../../../../docs/zircon_runtime/asset/assets/animation.md");

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
        animation_mod.contains("pub use error::{AnimationAssetError, AnimationAssetResult};"),
        "animation module should export AnimationAssetError/AnimationAssetResult"
    );

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
                !source.contains(forbidden),
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
        "asset/assets/animation/error.rs",
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

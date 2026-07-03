#[test]
fn review_f5_texture_loader_uses_typed_error() {
    let texture_loader = include_str!("../../../../../asset/load/texture.rs");
    let worker_pool = include_str!("../../../../../asset/pipeline/worker_pool.rs");
    let texture_tests = include_str!("../../../../../asset/tests/load/texture.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let worker_doc = include_str!("../../../../../../../docs/zircon_runtime/asset/worker_pool.md");

    for required in [
        "pub(crate) type TextureLoadResult<T> = std::result::Result<T, TextureLoadError>;",
        "pub(crate) enum TextureLoadError",
        "OpenImage {",
        "#[source]",
        "source: image::ImageError",
        "pub(crate) fn load_texture(source: &TextureSource) -> TextureLoadResult<CpuTexturePayload>",
        "pub(crate) fn decode_image_file(path: &str) -> TextureLoadResult<CpuTexturePayload>",
        "TextureLoadError::OpenImage",
    ] {
        assert!(
            texture_loader.contains(required),
            "F5 texture loader typed error owner should contain `{required}`"
        );
    }

    for forbidden in [
        "Result<CpuTexturePayload, String>",
        "format!(\"open image {path}: {error}\")",
    ] {
        assert!(
            !texture_loader.contains(forbidden),
            "texture loader should not keep lossy String error branch `{forbidden}`"
        );
    }

    for required in [
        "message: error.to_string()",
        "missing_image_file_reports_typed_texture_load_error",
        "TextureLoadError::OpenImage",
    ] {
        assert!(
            worker_pool.contains(required) || texture_tests.contains(required),
            "texture loader worker/test surface should contain `{required}`"
        );
    }

    for doc_anchor in [
        "F5 texture loader typed errors",
        "runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred",
        "review_f5_texture_loader_uses_typed_error",
        "TextureLoadError::OpenImage",
        "asset/load/texture.rs",
        "asset/tests/load/texture.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || worker_doc.contains(doc_anchor),
            "F5 texture loader docs should record `{doc_anchor}`"
        );
    }
}

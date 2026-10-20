//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn review_f5_texture_loader_uses_typed_error() {
    let texture_loader = include_str!("../../../../../asset/load/texture.rs");
    let worker_pool = include_str!("../../../../../asset/pipeline/worker_pool.rs");
    let worker_payload = include_str!("../../../../../asset/pipeline/worker_pool/payload.rs");
    assert!(worker_pool.contains("#[path = \"worker_pool/payload.rs\"]"));
    assert!(worker_pool.contains("mod payload;"));
    let texture_tests = include_str!("../../../../../asset/tests/load/texture.rs");
    let worker_doc =
        include_str!("../../../../../../../docs/crates/zircon_runtime/asset/worker_pool.md");

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
            worker_payload.contains(required) || texture_tests.contains(required),
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
            worker_doc.contains(doc_anchor),
            "F5 texture loader docs should record `{doc_anchor}`"
        );
    }
}

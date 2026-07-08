use super::*;

#[test]
fn runtime_15_review_guard_typed_error_child_rows_keep_representative_anchors() {
    let child_rows = [
        read_runtime_src(TYPED_ERROR_NATIVE_PLUGIN_ROWS_PATH),
        read_runtime_src(TYPED_ERROR_RUNTIME_SURFACE_ROWS_PATH),
        read_runtime_src(TYPED_ERROR_ASSET_SHADER_ROWS_PATH),
    ]
    .join("\n");

    assert_contains_all(
        "typed-error row-data children own representative rows",
        &child_rows,
        &[
            "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
            "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split",
            "Runtime 15 M3 scene world typed-error review guard child-owner split",
            "Runtime 15 M3 UI input typed-error review guard child-owner split",
            "Runtime 15 M3 asset records typed-error review guard child-owner split",
            "Runtime 15 M3 shader prewarm CLI typed-error review guard child-owner split",
        ],
    );
}

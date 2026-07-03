use super::*;

#[test]
fn runtime_15_foundation_topic_rows_are_child_owned() {
    let foundation_core_rows = read_runtime_src(FOUNDATION_CORE_ROWS_PATH);
    let foundation_typed_error_runtime_rows =
        read_runtime_src(FOUNDATION_TYPED_ERROR_RUNTIME_ROWS_PATH);
    let foundation_typed_error_plugin_rows =
        read_runtime_src(FOUNDATION_TYPED_ERROR_PLUGIN_ROWS_PATH);
    let foundation_typed_error_scene_asset_rows =
        read_runtime_src(FOUNDATION_TYPED_ERROR_SCENE_ASSET_ROWS_PATH);

    assert_contains_all(
        "Runtime 15 foundation core row child owns non-F5 foundation rows",
        &foundation_core_rows,
        &[
            "Runtime 15 F9 runtime prelude required type coverage",
            "Runtime 15 F12 dead-code review status sync",
            "Runtime 15 M1 graphics facade visibility review findings mirror",
            "Runtime 15 F13 provider registration shared owner",
            "Runtime 15 M1 animation manager folder-backed cutover",
        ],
    );
    assert_contains_all(
        "Runtime 15 foundation typed-error runtime child owns runtime typed-error rows",
        &foundation_typed_error_runtime_rows,
        &[
            "Runtime 15 F5 UI input surrounding-text error source",
            "Runtime 15 F5 shader prewarm CLI typed-error sweep",
            "Runtime 15 F5 dynamic API session typed errors",
        ],
    );
    assert_contains_all(
        "Runtime 15 foundation typed-error plugin child owns plugin typed-error rows",
        &foundation_typed_error_plugin_rows,
        &[
            "Runtime 15 F5 native plugin descriptor ABI typed errors",
            "Runtime 15 F5 native host API adapter typed errors",
            "Runtime 15 F5 native live-host runtime behavior typed errors",
        ],
    );
    assert_contains_all(
        "Runtime 15 foundation typed-error scene/asset child owns scene and asset typed-error rows",
        &foundation_typed_error_scene_asset_rows,
        &[
            "Runtime 15 F5 fixed world mutation typed errors",
            "Runtime 15 F5 asset meta typed errors",
            "Runtime 15 F5 mesh loader typed errors",
            "Runtime 15 F5 sound asset panic-free read helpers",
            "Runtime 15 F7 artifact cache JSON number typed errors",
        ],
    );
    let foundation_topic_row_count = [
        foundation_core_rows.as_str(),
        foundation_typed_error_runtime_rows.as_str(),
        foundation_typed_error_plugin_rows.as_str(),
        foundation_typed_error_scene_asset_rows.as_str(),
    ]
    .iter()
    .map(|source| runtime_15_row_count(source))
    .sum::<usize>();
    assert_eq!(
        73, foundation_topic_row_count,
        "foundation topic child owners should preserve all 73 Runtime 15 foundation status rows"
    );
}

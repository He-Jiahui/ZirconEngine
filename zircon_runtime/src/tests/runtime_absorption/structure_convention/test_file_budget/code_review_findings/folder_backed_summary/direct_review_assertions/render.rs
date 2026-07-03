use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

const DIRECT_REVIEW_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
const RENDER_DIRECT_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render.rs";
const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) fn assert_render_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    assert_contains_all(
        "render structure child owns F16 render_compiled_scene review guard",
        &sources.render_structure,
        &[
            "fn review_f16_compiled_scene_render_path_uses_split_owners",
            "bind_compiled_scene_graph_resources.rs",
            "execute_compiled_scene_graph_stages.rs",
            "submit_compiled_scene_frame.rs",
            "compiled_scene_render_split_review_guard_static_passed_cargo_deferred",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_render_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(RENDER_DIRECT_ASSERTIONS_CHILD);
    let sources = super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates render assertions to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/render.rs\"]",
            "mod render;",
            "render::assert_render_direct_sources_are_folder_backed",
        ],
    );
    for moved_guard in [
        concat!(
            "render structure child owns F16 render_compiled_scene ",
            "review guard"
        ),
        "review_f16_compiled_scene_render_path_uses_split_owners",
        "compiled_scene_render_split_review_guard_static_passed_cargo_deferred",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "render direct assertion `{moved_guard}` should stay in {RENDER_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "render direct assertion child owns render source checks",
        &child,
        &[
            "pub(super) fn assert_render_direct_sources_are_folder_backed",
            "render structure child owns F16 render_compiled_scene review guard",
            "fn review_f16_compiled_scene_render_path_uses_split_owners",
            "bind_compiled_scene_graph_resources.rs",
            "execute_compiled_scene_graph_stages.rs",
            "submit_compiled_scene_frame.rs",
            "compiled_scene_render_split_review_guard_static_passed_cargo_deferred",
        ],
    );

    assert_render_direct_sources_are_folder_backed(&sources);

    for (path, source) in [
        (DIRECT_REVIEW_ASSERTIONS_CHILD, parent.as_str()),
        (RENDER_DIRECT_ASSERTIONS_CHILD, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

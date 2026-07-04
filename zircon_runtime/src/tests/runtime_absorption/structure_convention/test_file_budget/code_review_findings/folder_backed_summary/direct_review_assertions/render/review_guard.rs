use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_render_compiled_scene_review_guard_is_child_owned(
    sources: &CodeReviewFindingsSources,
) {
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
fn runtime_15_code_review_findings_render_direct_assertions_guard_is_folder_backed() {
    let render_parent = read_runtime_src(RENDER_DIRECT_ASSERTIONS_CHILD);
    let child_blob = render_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

    assert_render_compiled_scene_review_guard_is_child_owned(&sources);
    budgets::assert_render_direct_assertions_children_line_budgets_are_current();
    for (_, child_path, child_guard) in RENDER_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            render_parent.contains(child_path),
            "render direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "render direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !render_parent
            .contains("render structure child owns F16 render_compiled_scene review guard"),
        "render.rs should delegate render structure review assertions to review_guard.rs"
    );
    assert_contains_all(
        "render direct assertions parent records folder-backed status",
        &render_parent,
        &[
            RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
            RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            RENDER_DIRECT_ASSERTIONS_STATUS_GUARD,
            RENDER_DIRECT_ASSERTIONS_BUDGET_GUARD,
        ],
    );
}

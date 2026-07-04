use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_render_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(RENDER_DIRECT_ASSERTIONS_CHILD);
    let child_blob = render_direct_assertion_child_source_blob();
    let sources = super::super::super::source_inventory::code_review_findings_sources();

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
        "render direct assertion parent owns child inventory",
        &child,
        &[
            "pub(super) fn assert_render_direct_sources_are_folder_backed",
            RENDER_DIRECT_ASSERTIONS_DELEGATION_CHILD,
            RENDER_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD,
            RENDER_DIRECT_ASSERTIONS_BUDGETS_CHILD,
            RENDER_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
            "runtime_15_code_review_findings_render_direct_assertions_are_child_owner",
            RENDER_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
            RENDER_DIRECT_ASSERTIONS_STATUS_GUARD,
        ],
    );

    assert_render_direct_sources_are_folder_backed(&sources);
    for (_, child_path, child_guard) in RENDER_DIRECT_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "render direct assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "render direct assertions child source blob should contain child guard {child_guard}"
        );
    }
    budgets::assert_render_direct_assertions_children_line_budgets_are_current();
}

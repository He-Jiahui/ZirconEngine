use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD);
    let child = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child_tree = direct_review_assertion_child_source_blob();
    let sources = super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "folder-backed summary delegates direct review assertions to child owner",
        &parent,
        &[
            "#[path = \"folder_backed_summary/direct_review_assertions.rs\"]",
            "mod direct_review_assertions;",
            "direct_review_assertions::assert_code_review_direct_sources_are_folder_backed",
        ],
    );
    for direct_review_guard in [
        concat!(
            "P0 robustness parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "F8 API convergence parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "render structure child owns F16 render_compiled_scene ",
            "review guard"
        ),
        concat!(
            "F12 dead-code child owns production suppression ",
            "review guard"
        ),
    ] {
        assert!(
            !parent.contains(direct_review_guard),
            "direct review guard `{direct_review_guard}` should stay in {DIRECT_REVIEW_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "direct-review assertion parent delegates focused guard children",
        &child,
        &[
            "#[path = \"direct_review_assertions/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"direct_review_assertions/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"direct_review_assertions/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"direct_review_assertions/f12.rs\"]",
            "mod f12;",
            "#[path = \"direct_review_assertions/f8.rs\"]",
            "mod f8;",
            "#[path = \"direct_review_assertions/p0.rs\"]",
            "mod p0;",
            "#[path = \"direct_review_assertions/render.rs\"]",
            "mod render;",
            "#[path = \"direct_review_assertions/root_parent.rs\"]",
            "mod root_parent;",
            "pub(super) fn assert_code_review_direct_sources_are_folder_backed",
            "CodeReviewFindingsSources",
            "f12::assert_f12_direct_sources_are_folder_backed",
            "f8::assert_f8_direct_sources_are_folder_backed",
            "p0::assert_p0_direct_sources_are_folder_backed",
            "render::assert_render_direct_sources_are_folder_backed",
            "root_parent::assert_code_review_root_parent_is_folder_backed",
            "direct_review_assertion_child_sources",
            "direct_review_assertion_child_source_blob",
        ],
    );
    assert_contains_all(
        "direct-review assertion guard children own delegated assertions",
        &child_tree,
        &[
            "runtime_15_code_review_findings_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_direct_assertions_children_are_child_owned",
            "runtime_15_code_review_findings_f12_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_render_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner",
        ],
    );
    for (_, child_path, anchor) in DIRECT_REVIEW_ASSERTIONS_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "direct-review assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "direct-review assertion child {child_path} should own anchor {anchor}"
        );
    }

    assert_code_review_direct_sources_are_folder_backed(&sources);

    for (path, source) in [
        (FOLDER_BACKED_SUMMARY_CHILD, parent),
        (DIRECT_REVIEW_ASSERTIONS_CHILD, child),
    ]
    .into_iter()
    .chain(direct_review_assertion_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

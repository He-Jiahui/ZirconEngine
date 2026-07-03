use super::super::super::super::*;
use super::*;

fn folder_backed_summary_direct_assertion_child_tree() -> String {
    [
        read_runtime_src(FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_DELEGATION_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_CHILD_OWNERSHIP_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_F12_DIRECT_ASSERTIONS_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_F8_DIRECT_ASSERTIONS_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_P0_DIRECT_ASSERTIONS_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_RENDER_DIRECT_ASSERTIONS_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_ROOT_PARENT_DIRECT_ASSERTIONS_CHILD_OWNER),
    ]
    .join("\n")
}

pub(super) fn assert_folder_backed_summary_direct_assertions_are_current() {
    let parent = read_runtime_src(FOLDER_BACKED_SUMMARY_STRUCTURE_CHILD_OWNER);
    let child_tree = folder_backed_summary_direct_assertion_child_tree();

    for direct_guard in [
        "folder-backed summary direct-assertions child keeps focused direct source checks",
        "folder-backed summary direct-assertions leaf children keep focused direct source checks",
        "review_priority_recommendation_tracks_current_remaining_work",
        "review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
    ] {
        assert!(
            !parent.contains(direct_guard),
            "direct assertion structure guard `{direct_guard}` should stay in {FOLDER_BACKED_SUMMARY_STRUCTURE_DIRECT_ASSERTIONS_CHILD_OWNER}"
        );
    }
    assert_contains_all(
        "folder-backed summary direct-assertions child keeps focused direct source checks",
        &child_tree,
        &[
            "fn runtime_15_code_review_findings_direct_assertions_are_child_owner",
            "fn runtime_15_code_review_findings_direct_assertions_children_are_child_owned",
            "fn runtime_15_code_review_findings_direct_assertions_guard_folder_backed_status_is_current",
            "pub(super) fn assert_code_review_direct_sources_are_folder_backed",
            "CodeReviewFindingsSources",
            "#[path = \"direct_review_assertions/delegation.rs\"]",
            "#[path = \"direct_review_assertions/child_ownership.rs\"]",
            "#[path = \"direct_review_assertions/status_mirrors.rs\"]",
            "#[path = \"direct_review_assertions/f12.rs\"]",
            "#[path = \"direct_review_assertions/f8.rs\"]",
            "#[path = \"direct_review_assertions/p0.rs\"]",
            "#[path = \"direct_review_assertions/render.rs\"]",
            "f12::assert_f12_direct_sources_are_folder_backed",
            "f8::assert_f8_direct_sources_are_folder_backed",
            "p0::assert_p0_direct_sources_are_folder_backed",
            "render::assert_render_direct_sources_are_folder_backed",
            "P0 robustness parent only mounts focused child review guard owners",
            "F8 API convergence parent only mounts focused child review guard owners",
            "render structure child owns F16 render_compiled_scene review guard",
            "F12 dead-code child owns production suppression review guard",
            "review_priority_recommendation_tracks_current_remaining_work",
            "review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_structure_guard_folder_backed_summary_direct_assertions_are_child_owned(
) {
    assert_folder_backed_summary_direct_assertions_are_current();
}

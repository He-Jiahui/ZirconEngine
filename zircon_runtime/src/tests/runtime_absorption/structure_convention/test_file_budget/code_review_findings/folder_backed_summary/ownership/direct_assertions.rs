use super::super::super::super::*;
use super::*;

pub(super) fn assert_folder_backed_direct_review_assertion_children_are_current() {
    let direct_review_assertions_child = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let direct_review_assertion_child_sources =
        direct_review_assertions::direct_review_assertion_child_source_blob();

    assert_contains_all(
        "folder-backed direct-review assertions child mounts focused direct source checks",
        &direct_review_assertions_child,
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
        ],
    );
    assert_contains_all(
        "folder-backed direct-review assertion child tree owns delegated guard tests",
        &direct_review_assertion_child_sources,
        &[
            "fn runtime_15_code_review_findings_direct_assertions_are_child_owner",
            "fn runtime_15_code_review_findings_direct_assertions_children_are_child_owned",
            "runtime_15_code_review_findings_f12_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_render_direct_assertions_are_child_owner",
            "runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner",
        ],
    );
    assert_direct_assertion_route_parent_has_entry_points(
        F12_DIRECT_ASSERTIONS_CHILD,
        &[
            "pub(super) fn assert_f12_direct_sources_are_folder_backed",
            "pub(super) fn f12_direct_assertion_child_source_blob",
        ],
    );
    assert_direct_assertion_route_parent_has_entry_points(
        F8_DIRECT_ASSERTIONS_CHILD,
        &[
            "pub(super) fn assert_f8_direct_sources_are_folder_backed",
            "pub(super) fn f8_direct_assertion_child_source_blob",
        ],
    );
    assert_direct_assertion_route_parent_has_entry_points(
        P0_DIRECT_ASSERTIONS_CHILD,
        &[
            "pub(super) fn assert_p0_direct_sources_are_folder_backed",
            "pub(super) fn p0_direct_assertion_child_source_blob",
        ],
    );
    assert_direct_assertion_route_parent_has_entry_points(
        RENDER_DIRECT_ASSERTIONS_CHILD,
        &[
            "pub(super) fn assert_render_direct_sources_are_folder_backed",
            "pub(super) fn render_direct_assertion_child_source_blob",
        ],
    );
    assert_direct_assertion_route_parent_has_entry_points(
        ROOT_PARENT_DIRECT_ASSERTIONS_CHILD,
        &[
            "pub(super) fn assert_code_review_root_parent_is_folder_backed",
            "pub(super) fn root_parent_direct_assertion_child_source_blob",
        ],
    );
}

fn assert_direct_assertion_route_parent_has_entry_points(path: &str, anchors: &[&str]) {
    let source = read_runtime_src(path);

    assert_contains_all(
        "folder-backed direct assertion route parent owns expected helper entry points",
        &source,
        anchors,
    );
}

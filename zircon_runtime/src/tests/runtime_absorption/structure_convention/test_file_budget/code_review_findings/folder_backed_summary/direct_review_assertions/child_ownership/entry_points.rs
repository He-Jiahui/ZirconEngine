use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_direct_review_child_entry_points_are_current() {
    let f12_child = read_runtime_src(F12_DIRECT_ASSERTIONS_CHILD);
    let f12_child_blob = super::super::f12::f12_direct_assertion_child_source_blob();
    let f8_child = read_runtime_src(F8_DIRECT_ASSERTIONS_CHILD);
    let f8_child_blob = super::super::f8::f8_direct_assertion_child_source_blob();
    let p0_child = read_runtime_src(P0_DIRECT_ASSERTIONS_CHILD);
    let p0_child_blob = super::super::p0::p0_direct_assertion_child_source_blob();
    let render_child = read_runtime_src(RENDER_DIRECT_ASSERTIONS_CHILD);
    let render_child_blob = super::super::render::render_direct_assertion_child_source_blob();
    let root_parent_child = read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_CHILD);
    let root_parent_child_blob =
        super::super::root_parent::root_parent_direct_assertion_child_source_blob();

    assert_contains_all(
        "F12 direct assertion child owns F12 route/helper entry points",
        &f12_child,
        &["pub(super) fn assert_f12_direct_sources_are_folder_backed"],
    );
    assert_contains_all(
        "F12 direct assertion nested children own F12 source check entry points",
        &f12_child_blob,
        &["fn runtime_15_code_review_findings_f12_direct_assertions_are_child_owner"],
    );
    assert_contains_all(
        "F8 direct assertion child owns F8 route/helper entry points",
        &f8_child,
        &[
            "pub(super) fn assert_f8_direct_sources_are_folder_backed",
            "runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
        ],
    );
    assert_contains_all(
        "F8 direct assertion nested children own F8 source check entry points",
        &f8_child_blob,
        &["fn runtime_15_code_review_findings_f8_direct_assertions_are_child_owner"],
    );
    assert_contains_all(
        "P0 direct assertion child owns P0 route/helper entry points",
        &p0_child,
        &[
            "pub(super) fn assert_p0_direct_sources_are_folder_backed",
            "runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
        ],
    );
    assert_contains_all(
        "P0 direct assertion nested children own P0 source check entry points",
        &p0_child_blob,
        &["fn runtime_15_code_review_findings_p0_direct_assertions_are_child_owner"],
    );
    assert_contains_all(
        "render direct assertion child owns render route/helper entry points",
        &render_child,
        &["pub(super) fn assert_render_direct_sources_are_folder_backed"],
    );
    assert_contains_all(
        "render direct assertion nested children own render source check entry points",
        &render_child_blob,
        &["fn runtime_15_code_review_findings_render_direct_assertions_are_child_owner"],
    );
    assert_contains_all(
        "root-parent direct assertion parent owns root parent route/helper entry points",
        &root_parent_child,
        &["pub(super) fn assert_code_review_root_parent_is_folder_backed"],
    );
    assert_contains_all(
        "root-parent direct assertion nested children own root parent check entry points",
        &root_parent_child_blob,
        &["fn runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner"],
    );
}

use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_direct_assertions_children_are_child_owned() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let f12_child = read_runtime_src(F12_DIRECT_ASSERTIONS_CHILD);
    let f8_child = read_runtime_src(F8_DIRECT_ASSERTIONS_CHILD);
    let p0_child = read_runtime_src(P0_DIRECT_ASSERTIONS_CHILD);
    let render_child = read_runtime_src(RENDER_DIRECT_ASSERTIONS_CHILD);
    let root_parent_child = read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_CHILD);
    let sources = super::super::source_inventory::code_review_findings_sources();

    for f12_guard in [
        concat!(
            "F12 dead-code child owns production suppression ",
            "review guard"
        ),
        concat!(
            "review_f12_runtime_production_dead_code_",
            "suppression_is_globally_gated"
        ),
    ] {
        assert!(
            !parent.contains(f12_guard),
            "F12 direct assertion `{f12_guard}` should stay in {F12_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    for render_guard in [
        concat!(
            "render structure child owns F16 render_compiled_scene ",
            "review guard"
        ),
        concat!(
            "review_f16_compiled_scene_render_path_uses_",
            "split_owners"
        ),
    ] {
        assert!(
            !parent.contains(render_guard),
            "render direct assertion `{render_guard}` should stay in {RENDER_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    for root_parent_guard in [
        concat!(
            "code review findings parent mounts ",
            "folder-backed children"
        ),
        concat!("review_f5_world_spawn_bundle_surface_uses_", "scene_error"),
        concat!("review_d13_importer_runtime_manifests_use_", "sdk_builder"),
    ] {
        assert!(
            !parent.contains(root_parent_guard),
            "root-parent direct assertion `{root_parent_guard}` should stay in {ROOT_PARENT_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    for f8_guard in [
        concat!(
            "F8 API convergence parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "review_f8_runtime_plugin_descriptor_public_",
            "constructor_is_retired"
        ),
    ] {
        assert!(
            !parent.contains(f8_guard),
            "F8 direct assertion `{f8_guard}` should stay in {F8_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    for p0_guard in [
        concat!(
            "P0 robustness parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "P0 priority recommendation child owns current remaining-work ",
            "review guard"
        ),
        concat!(
            "review_priority_recommendation_",
            "tracks_current_remaining_work"
        ),
    ] {
        assert!(
            !parent.contains(p0_guard),
            "P0 direct assertion `{p0_guard}` should stay in {P0_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "F12 direct assertion child owns F12 direct source check entry points",
        &f12_child,
        &[
            "fn runtime_15_code_review_findings_f12_direct_assertions_are_child_owner",
            "pub(super) fn assert_f12_direct_sources_are_folder_backed",
        ],
    );
    assert_contains_all(
        "F8 direct assertion child owns F8 direct source check entry points",
        &f8_child,
        &[
            "fn runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
            "pub(super) fn assert_f8_direct_sources_are_folder_backed",
        ],
    );
    assert_contains_all(
        "P0 direct assertion child owns P0 direct source check entry points",
        &p0_child,
        &[
            "fn runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
            "pub(super) fn assert_p0_direct_sources_are_folder_backed",
        ],
    );
    assert_contains_all(
        "render direct assertion child owns render source check entry points",
        &render_child,
        &[
            "fn runtime_15_code_review_findings_render_direct_assertions_are_child_owner",
            "pub(super) fn assert_render_direct_sources_are_folder_backed",
        ],
    );
    assert_contains_all(
        "root-parent direct assertion child owns root parent check entry points",
        &root_parent_child,
        &[
            "fn runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner",
            "pub(super) fn assert_code_review_root_parent_is_folder_backed",
        ],
    );

    assert_code_review_direct_sources_are_folder_backed(&sources);

    for (path, source) in [(DIRECT_REVIEW_ASSERTIONS_CHILD, parent)]
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

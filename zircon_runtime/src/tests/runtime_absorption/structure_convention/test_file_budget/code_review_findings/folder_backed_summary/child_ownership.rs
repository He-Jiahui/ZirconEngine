use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_folder_backed_summary_children_are_child_owned() {
    let parent = read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD);
    let direct_review_assertions_child = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let direct_review_assertion_child_sources =
        direct_review_assertions::direct_review_assertion_child_source_blob();
    let f12_direct_assertions_child = read_runtime_src(F12_DIRECT_ASSERTIONS_CHILD);
    let f8_direct_assertions_child = read_runtime_src(F8_DIRECT_ASSERTIONS_CHILD);
    let p0_direct_assertions_child = read_runtime_src(P0_DIRECT_ASSERTIONS_CHILD);
    let render_direct_assertions_child = read_runtime_src(RENDER_DIRECT_ASSERTIONS_CHILD);
    let root_parent_direct_assertions_child = read_runtime_src(ROOT_PARENT_DIRECT_ASSERTIONS_CHILD);
    let source_inventory_child_sources = source_inventory::source_inventory_child_source_blob();

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
            "fn runtime_15_code_review_findings_direct_assertions_guard_folder_backed_status_is_current",
        ],
    );
    assert_contains_all(
        "folder-backed F12 direct assertions child owns F12 source check entry points",
        &f12_direct_assertions_child,
        &[
            "fn runtime_15_code_review_findings_f12_direct_assertions_are_child_owner",
            "pub(super) fn assert_f12_direct_sources_are_folder_backed",
            F12_DIRECT_ASSERTIONS_CHILD,
        ],
    );
    assert_contains_all(
        "folder-backed F8 direct assertions child owns F8 source check entry points",
        &f8_direct_assertions_child,
        &[
            "fn runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
            "pub(super) fn assert_f8_direct_sources_are_folder_backed",
            F8_DIRECT_ASSERTIONS_CHILD,
        ],
    );
    assert_contains_all(
        "folder-backed P0 direct assertions child owns P0 source check entry points",
        &p0_direct_assertions_child,
        &[
            "fn runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
            "pub(super) fn assert_p0_direct_sources_are_folder_backed",
            P0_DIRECT_ASSERTIONS_CHILD,
        ],
    );
    assert_contains_all(
        "folder-backed render direct assertions child owns render source check entry points",
        &render_direct_assertions_child,
        &[
            "fn runtime_15_code_review_findings_render_direct_assertions_are_child_owner",
            "pub(super) fn assert_render_direct_sources_are_folder_backed",
            RENDER_DIRECT_ASSERTIONS_CHILD,
        ],
    );
    assert_contains_all(
        "folder-backed root-parent direct assertions child owns root parent check entry points",
        &root_parent_direct_assertions_child,
        &[
            "fn runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner",
            "pub(super) fn assert_code_review_root_parent_is_folder_backed",
            ROOT_PARENT_DIRECT_ASSERTIONS_CHILD,
        ],
    );
    for source_inventory_guard in [
        concat!("let ", "f8_api_convergence ="),
        concat!("let ", "p0_robustness ="),
        concat!(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/",
            "descriptor_builder/scaffold.rs"
        ),
        concat!(
            "tests/runtime_absorption/code_review_findings/p0_robustness/",
            "native_fixture/sdk_macro_manifest.rs"
        ),
    ] {
        assert!(
            !parent.contains(source_inventory_guard),
            "source inventory guard `{source_inventory_guard}` should stay in {SOURCE_INVENTORY_CHILD}"
        );
    }
    assert_contains_all(
        "folder-backed source inventory child owns source reads and helper counts",
        &source_inventory_child_sources,
        &[
            "fn runtime_15_code_review_findings_source_inventory_is_child_owner",
            "struct CodeReviewFindingsSources",
            "pub(super) fn code_review_findings_sources",
            "pub(super) fn assert_code_review_findings_line_budgets",
            "fn direct_review_guard_count",
            concat!(
                "tests/runtime_absorption/code_review_findings/f8_api_convergence/",
                "descriptor_builder/scaffold.rs"
            ),
            concat!(
                "tests/runtime_absorption/code_review_findings/p0_robustness/",
                "native_fixture/sdk_macro_manifest.rs"
            ),
            "tests/runtime_absorption/code_review_findings/render_structure.rs",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
        ],
    );
}

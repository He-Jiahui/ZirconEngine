use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

const DIRECT_REVIEW_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
const F12_DIRECT_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs";
const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) fn assert_f12_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    assert_contains_all(
        "F12 dead-code child owns production suppression review guard",
        &sources.f12_dead_code,
        &[
            "fn review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
            "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
            "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
            "Runtime production `allow(dead_code)` sweep is globally gated",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_f12_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(F12_DIRECT_ASSERTIONS_CHILD);
    let sources = super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates F12 assertions to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/f12.rs\"]",
            "mod f12;",
            "f12::assert_f12_direct_sources_are_folder_backed",
        ],
    );
    for moved_guard in [
        concat!(
            "F12 dead-code child owns production suppression ",
            "review guard"
        ),
        "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
        "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
        "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "F12 direct assertion `{moved_guard}` should stay in {F12_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "F12 direct assertion child owns F12 source checks",
        &child,
        &[
            "pub(super) fn assert_f12_direct_sources_are_folder_backed",
            "F12 dead-code child owns production suppression review guard",
            "fn review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
            "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
            "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
            "Runtime production `allow(dead_code)` sweep is globally gated",
        ],
    );

    assert_f12_direct_sources_are_folder_backed(&sources);

    for (path, source) in [
        (DIRECT_REVIEW_ASSERTIONS_CHILD, parent.as_str()),
        (F12_DIRECT_ASSERTIONS_CHILD, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

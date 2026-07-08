use super::super::super::super::super::super::super::super::*;
use super::super::super::super::super::sources::typed_error_status_row_source;
use super::super::super::super::review_guard_paths::{
    REVIEW_GUARD_DATE_MAP_PATH, REVIEW_GUARD_STATUS_MAP_PATH,
    REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH, REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH,
};
use super::super::super::super::root_paths::{
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_PATHS_CHILDREN_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_STATUS_CURRENT_CHILDREN_CHILD,
};
use super::super::super::super::status_slices::{
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_DATE,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_GUARD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_ID,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_NAME,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD_SPLIT_DATE,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD_SPLIT_ID,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD_SPLIT_NAME,
};
use super::source_tree::typed_error_paths_status_current_sources_guard_children;

const TYPED_ERROR_STATUS_DOC_PATHS_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/paths_inventory_rows.rs";
const TYPED_ERROR_STATUS_DOC_PATHS_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/status_doc_rows/paths_inventory_rows.rs";

pub(super) fn assert_sources_child_split_status_is_current() {
    let anchors = [
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_NAME,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_ID,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_PATHS_CHILDREN_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_STATUS_CURRENT_CHILDREN_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_GUARD,
        "Cargo gate deferred",
    ];

    assert_status_documents_contain(
        "typed-error paths status-current sources child split",
        &anchors,
    );
    assert_status_maps_contain(
        "typed-error paths status-current sources child split",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_NAME,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_ID,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_CHILD_SPLIT_DATE,
    );
}

pub(super) fn assert_sources_guard_folder_backed_status_is_current() {
    let anchors = [
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD_SPLIT_NAME,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD_SPLIT_ID,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_SOURCE_SPLIT_CHILD,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD,
        "Cargo gate deferred",
    ];

    assert_status_documents_contain(
        "typed-error paths status-current sources guard folder-backed split",
        &anchors,
    );
    let status_rows = typed_error_status_row_source();
    for path in typed_error_paths_status_current_sources_guard_children() {
        assert!(
            status_rows.contains(path),
            "typed-error paths status-current sources guard row should include child source {path}"
        );
    }
    assert_status_maps_contain(
        "typed-error paths status-current sources guard folder-backed split",
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD_SPLIT_NAME,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD_SPLIT_ID,
        TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SOURCES_GUARD_SPLIT_DATE,
    );
}

fn assert_status_documents_contain(label: &str, anchors: &[&str]) {
    let status_rows = typed_error_status_row_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (source_label, source) in [
        ("typed-error row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02 plan", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(&format!("{label} {source_label}"), source, anchors);
    }
}

fn assert_status_maps_contain(label: &str, split_name: &str, split_id: &str, split_date: &str) {
    let status_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH),
        read_runtime_src(TYPED_ERROR_STATUS_DOC_PATHS_STATUS_MAP_PATH)
    );
    let date_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH),
        read_runtime_src(TYPED_ERROR_STATUS_DOC_PATHS_DATE_MAP_PATH)
    );

    assert_contains_all(
        &format!("M3 review status map records {label}"),
        &status_map,
        &[split_name, split_id],
    );
    assert_contains_all(
        &format!("M3 review date map records {label}"),
        &date_map,
        &[split_name, split_date],
    );
}

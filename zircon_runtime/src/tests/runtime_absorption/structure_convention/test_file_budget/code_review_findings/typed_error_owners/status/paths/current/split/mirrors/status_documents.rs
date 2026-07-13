use super::super::super::super::super::super::super::super::*;
use super::super::super::super::super::sources::typed_error_status_row_source;

pub(super) fn assert_status_documents_contain(label: &str, anchors: &[&str]) {
    let status_rows = typed_error_status_row_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (source_label, source) in [
        ("typed-error row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(&format!("{label} {source_label}"), source, anchors);
    }
}

pub(super) fn assert_frameworks_contain(label: &str, anchors: &[&str]) {
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    assert_contains_all(label, &frameworks_02, anchors);
}

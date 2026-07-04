use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_code_review_root_parent_mounts_are_folder_backed(
    sources: &CodeReviewFindingsSources,
) {
    assert_contains_all(
        "code review findings parent mounts folder-backed children",
        &sources.parent,
        &[
            "mod f12_dead_code;",
            "mod f8_api_convergence;",
            "mod late_api_cleanup;",
            "mod p0_robustness;",
            "mod plugin_importer_dx;",
            "mod render_structure;",
            "mod typed_error_convergence;",
        ],
    );
    assert_eq!(
        sources.parent.matches("#[test]").count(),
        0,
        "code_review_findings.rs should only mount child test owners"
    );
}

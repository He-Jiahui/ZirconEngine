const PARENT_SOURCE: &str = include_str!("../ui_architecture.rs");
const ARCHITECTURE_BOUNDARIES_SOURCE: &str = include_str!("architecture_boundaries.rs");
const LEGACY_RENAMES_SOURCE: &str = include_str!("legacy_renames.rs");
const MIRROR_DOCS_SOURCE: &str = include_str!("mirror_docs.rs");
const SUPPORT_SOURCE: &str = include_str!("support.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const FRAMEWORKS_02_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md");
const RUNTIME_15_PLAN: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
);
const RUNTIME_INDEX: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
const STRUCTURE_CONVENTION_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
const REVIEW_FINDINGS_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
#[rustfmt::skip]
const NUMBERED_STATUS_RECORDS: &str = concat!(
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/09/2026-07-09-ui-subsystem-architecture-output-records.md"),
    include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"),
    include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
    include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
    include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md")
);

#[test]
fn runtime_15_ui_architecture_route_owner_is_folder_backed() {
    assert_contains_all(
        "ui_architecture route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"ui_architecture/support.rs\"]",
            "#[path = \"ui_architecture/architecture_boundaries.rs\"]",
            "#[path = \"ui_architecture/legacy_renames.rs\"]",
            "#[path = \"ui_architecture/mirror_docs.rs\"]",
            "#[path = \"ui_architecture/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owners_are_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "ui_architecture.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "fn repo_root",
        "fn read_repo_file",
        "top_level_entry_names(",
        "rust_files_under(",
        "production_ui_file(",
        "matching_line_count(",
        "files_with_matching_line(",
        "std::fs::read_to_string",
        "std::fs::read_dir",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "ui_architecture.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "support child",
        SUPPORT_SOURCE,
        &[
            "pub(super) fn read_repo_file",
            "pub(super) fn top_level_entry_names",
            "pub(super) fn rust_files_under",
            "pub(super) fn production_ui_file",
            "pub(super) fn matching_line_count",
            "pub(super) fn files_with_matching_line",
        ],
    );
    assert_contains_all(
        "architecture boundaries child",
        ARCHITECTURE_BOUNDARIES_SOURCE,
        &[
            "runtime_09_ui_architecture_doc_records_current_boundaries",
            "runtime_09_ui_architecture_baselines_match_current_source_scan",
            "runtime_09_taffy_layout_pass_order_uses_bridge_authority",
            "use super::support::{",
        ],
    );
    assert_contains_all(
        "legacy renames child",
        LEGACY_RENAMES_SOURCE,
        &[
            "runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt",
            "runtime_09_ui_input_events_route_through_single_dispatch_authority",
            "use super::support::read_repo_file;",
        ],
    );
    assert_contains_all(
        "mirror docs child",
        MIRROR_DOCS_SOURCE,
        &["runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts"],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 12),
        ("support child", SUPPORT_SOURCE, 120),
        (
            "architecture boundaries child",
            ARCHITECTURE_BOUNDARIES_SOURCE,
            540,
        ),
        ("legacy renames child", LEGACY_RENAMES_SOURCE, 560),
        ("mirror docs child", MIRROR_DOCS_SOURCE, 140),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 210),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    for (label, source) in [
        ("Frameworks 02 plan", FRAMEWORKS_02_PLAN),
        ("Runtime 15 plan", RUNTIME_15_PLAN),
        ("runtime index", RUNTIME_INDEX),
        ("structure convention plan", STRUCTURE_CONVENTION_PLAN),
        ("review findings plan", REVIEW_FINDINGS_PLAN),
        ("module convention doc", MODULE_CONVENTION_DOC),
    ] {
        assert!(
            source.contains(
                "runtime_15_ui_architecture_route_owner_split_static_passed_cargo_deferred"
            ) || NUMBERED_STATUS_RECORDS.contains(
                "runtime_15_ui_architecture_route_owner_split_static_passed_cargo_deferred"
            ),
            "{label} should mirror the ui_architecture route-owner split status"
        );
    }
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "ui_architecture/support.rs",
            "ui_architecture/split_layout.rs",
            "runtime_15_ui_architecture_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 plan",
        NUMBERED_STATUS_RECORDS,
        &[
            "frameworks_02_m3_ui_architecture_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 ui-architecture route-owner split",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain split anchor `{anchor}`"
        );
    }
}

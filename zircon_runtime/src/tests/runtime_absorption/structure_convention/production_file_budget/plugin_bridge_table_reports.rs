use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_plugin_bridge_table_reports_are_child_owner() {
    let parent = read_runtime_src("plugin/bridge/table.rs");
    let reports = read_runtime_src("plugin/bridge/table/reports.rs");
    let bridge_root = read_runtime_src("plugin/bridge.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "plugin bridge table parent delegates diagnostics/report DTOs",
        &parent,
        &[
            "mod reports;",
            "pub use self::reports::{",
            "BridgeDiagnosticsMatrix",
            "BridgeOwnerTransitionReport",
            "BridgeTableDiagnosticsSummary",
            "pub struct BridgeEntry",
            "pub struct FrozenBridgeTable",
            "fn snapshot_for_entry(",
            "BridgeInterfaceSnapshot {",
        ],
    );
    for moved_owner in [
        "pub enum BridgeInterfaceStatus",
        "pub struct BridgeInterfaceSnapshot",
        "pub struct BridgeTableDiagnosticsSummary",
        "pub struct BridgeDiagnosticsMatrix",
        "pub struct BridgeOwnerTransitionReport",
        "pub enum BridgeOwnerTransitionMode",
        "impl fmt::Debug for InterfaceExport",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "plugin/bridge/table.rs should delegate {moved_owner} to plugin/bridge/table/reports.rs"
        );
    }
    assert_contains_all(
        "plugin bridge table report child owns public diagnostics DTOs",
        &reports,
        &[
            "impl fmt::Debug for super::InterfaceExport",
            "pub enum BridgeInterfaceStatus",
            "pub struct BridgeInterfaceSnapshot",
            "pub struct BridgeTableDiagnosticsSummary",
            "pub struct BridgeDiagnosticsMatrix",
            "pub struct BridgeOwnerTransitionReport",
            "pub enum BridgeOwnerTransitionMode",
            "pub(super) fn from_installed_entry",
            "pub(super) fn record_snapshot",
            "pub(super) fn from_rows",
        ],
    );
    assert_contains_all(
        "plugin bridge root still re-exports table report DTOs",
        &bridge_root,
        &[
            "pub use table::{",
            "BridgeDiagnosticsMatrix",
            "BridgeInterfaceSnapshot",
            "BridgeInterfaceStatus",
            "BridgeOwnerTransitionMode",
            "BridgeOwnerTransitionReport",
            "BridgeTableDiagnosticsSummary",
            "FrozenBridgeTable",
            "InterfaceExport",
        ],
    );

    for (path, source) in [
        ("plugin/bridge/table.rs", parent.as_str()),
        ("plugin/bridge/table/reports.rs", reports.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("plugin bridge doc", plugin_bridge_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 plugin bridge table diagnostics owner split",
                "runtime_15_plugin_bridge_table_diagnostics_owner_split_static_passed_cargo_deferred",
                "plugin/bridge/table.rs",
                "plugin/bridge/table/reports.rs",
                "runtime_15_plugin_bridge_table_reports_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status/date maps record plugin bridge table diagnostics owner split",
        &format!("{status_map}\n{date_map}"),
        &[
            "Runtime 15 M4 plugin bridge table diagnostics owner split",
            "runtime_15_plugin_bridge_table_diagnostics_owner_split_static_passed_cargo_deferred",
            "2026-07-01",
        ],
    );
}

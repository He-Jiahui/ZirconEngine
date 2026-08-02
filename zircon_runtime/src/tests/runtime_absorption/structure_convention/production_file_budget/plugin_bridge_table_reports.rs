use super::{assert_contains_all, assert_contains_all_exact, read_repo, read_runtime_src};

#[test]
fn runtime_15_plugin_bridge_table_reports_are_child_owner() {
    let parent = read_runtime_src("plugin/bridge/table.rs");
    let reports = read_runtime_src("plugin/bridge/table/reports.rs");
    let bridge_root = read_runtime_src("plugin/bridge.rs");
    let neutral_bridge = read_runtime_src("core/framework/bridge/mod.rs");
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");
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
            "pub struct BridgeInterfaceSnapshot",
            "pub struct BridgeTableDiagnosticsSummary",
            "pub struct BridgeDiagnosticsMatrix",
            "pub struct BridgeOwnerTransitionReport",
            "pub(super) fn record_snapshot",
            "pub(super) fn from_rows",
        ],
    );
    assert_contains_all(
        "neutral bridge owner holds cross-domain slot and lifecycle contracts",
        &neutral_bridge,
        &[
            "pub enum BridgeInterfaceStatus",
            "pub enum BridgeOwnerTransitionMode",
            "pub trait BridgeInvocationTable",
            "fn resolve_interface_slot(",
            "fn interface_status_at(",
        ],
    );
    assert_contains_all(
        "plugin bridge root exports plugin-owned table/report DTOs only",
        &bridge_root,
        &[
            "pub use table::{",
            "BridgeDiagnosticsMatrix",
            "BridgeInterfaceSnapshot",
            "BridgeOwnerTransitionReport",
            "BridgeTableDiagnosticsSummary",
            "FrozenBridgeTable",
            "InterfaceExport",
            "pub use weak::{BridgeGuard, WeakBridge};",
        ],
    );
    assert!(!bridge_root.contains("BridgeInterfaceStatus"));
    assert!(!bridge_root.contains("BridgeOwnerTransitionMode"));

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
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all_exact(
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

use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 07 owner-budget evidence drift resync",
        &[
            "`large_file_ownership_gate`",
            "38 hotspots",
            "runtime-other=14",
            "`runtime_absorption::performance_hotspots`",
        ],
    ),
    (
        "Runtime 07 owner-budget 38-hotspot 漂移同步",
        &[
            "`large_file_hotspot_count = 38`",
            "`runtime-framework-render=2`",
            "`runtime-other=14`",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 37-hotspot 漂移同步",
        &[
            "`large_file_hotspot_count = 37`",
            "`runtime-other=13`",
            "`hotspot_guard_anchor_count = 20`",
            "standalone `status_output_tables.rs` 2/2",
        ],
    ),
    (
        "Runtime 07 owner-budget 37-hotspot 再同步",
        &[
            "`large_file_hotspot_count = 37`",
            "`runtime-other=12`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=37",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 38-hotspot 回漂同步",
        &[
            "`large_file_hotspot_count = 38`",
            "`runtime-framework-render=2`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=38",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 39-hotspot 漂移同步",
        &[
            "`large_file_hotspot_count = 39`",
            "`runtime-framework-render=3`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=39",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 42-hotspot 漂移同步",
        &[
            "`large_file_hotspot_count = 42`",
            "`runtime-other=15`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=42",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget current doc mirror fix",
        &[
            "hotspot_inventory.md",
            "M0 review",
            "42 hotspots / 5 migration-debt owner groups / 0 unclassified hotspots",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 36-hotspot navigation split sync",
        &[
            "`large_file_hotspot_count = 36`",
            "`runtime-other=12`",
            "runtime_07_navigation_runtime_owner_split_reduces_owner_budget_hotspot_count",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 30-hotspot current audit sync",
        &[
            "`large_file_hotspot_count = 30`",
            "`runtime-other=13`",
            "`editor-retained-host=3`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=30",
            "standalone `performance_hotspots.rs` 6/6",
            "extract/ecs_query/performance profiling/FPS Cargo gates",
        ],
    ),
    (
        "Runtime 07 owner-budget 0-hotspot current audit sync",
        &[
            "`large_file_m1_gate_status = classified-and-clear`",
            "`large_file_hotspot_count = 0`",
            "`large_file_migration_debt_count = 0`",
            "`large_file_owner_class_count = 0`",
            "`large_file_unclassified_hotspot_count = 0`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=0",
            "standalone `performance_hotspots.rs` exact owner-budget guards",
            "extract/ecs_query/performance profiling/FPS Cargo gates",
        ],
    ),
];

type Slice = super::Slice;

pub(super) const SUBPLAN_MAP_SYNC: Slice = (
    "Runtime 15 M3 runtime index subplan map 01-15 sync",
    &[
        "runtime_15_runtime_index_subplan_map_01_15_sync_static_passed_cargo_deferred",
        "docs/plans/zircon_runtime/runtime/index.md",
        "14-runtime-module-family-closeout.md",
        "15-code-structure-and-module-conventions.md",
        "EXPECTED_SUBPLAN_COUNT = 15",
        "runtime_15_runtime_index_subplan_map_covers_01_15_status_locked",
    ],
);

pub(super) const PROBLEM_ROW_PARSER_SYNC: Slice = (
    "Runtime 15 M3 runtime index problem-row parser P01-P17 sync",
    &[
        "runtime_15_runtime_index_problem_row_parser_p01_p17_sync_static_passed_cargo_deferred",
        "runtime_plan_status_sources.py",
        "runtime_index_problem_rows",
        "EXPECTED_PROBLEM_ROW_COUNT = 17",
        "runtime_15_runtime_index_problem_row_parser_covers_p01_p17_status_locked",
    ],
);

const PARENT_SOURCE: &str = include_str!("../script_absorption.rs");
const LEGACY_CRATE_SOURCE: &str = include_str!("legacy_crate.rs");
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
const SESSION_NOTE: &str = include_str!(
    "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
);
const STATUS_ROW_DATA: &str = include_str!(
    "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests/runtime_absorption_platform_rows.rs"
);
const STATUS_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/core_route_rows.rs"
);
const DATE_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/core_route_rows.rs"
);
#[rustfmt::skip]
const NUMBERED_STATUS_RECORDS: &str = concat!(
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/13/2026-07-09-script-binding-and-reflection-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md")
);

#[test]
fn runtime_15_script_absorption_route_owner_is_folder_backed() {
    assert_contains_all(
        "script_absorption route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"script_absorption/legacy_crate.rs\"]",
            "#[path = \"script_absorption/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owner_is_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "script_absorption.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "zircon_script",
        "std::fs::read_to_string",
        "CARGO_MANIFEST_DIR",
        "include_str!(",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "script_absorption.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owner_is_focused() {
    assert_contains_all(
        "legacy crate child",
        LEGACY_CRATE_SOURCE,
        &[
            "script_subsystem_is_physically_absorbed_into_runtime_crate",
            "src/script/mod.rs",
            "pub use zircon_script::*",
            "../zircon_script/Cargo.toml",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 6),
        ("legacy crate child", LEGACY_CRATE_SOURCE, 40),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 180),
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
        ("session note", SESSION_NOTE),
        ("status row data", STATUS_ROW_DATA),
        ("status map", STATUS_MAP),
    ] {
        assert!(
            source.contains(
                "runtime_15_script_absorption_route_owner_split_static_passed_cargo_deferred"
            ) || NUMBERED_STATUS_RECORDS.contains(
                "runtime_15_script_absorption_route_owner_split_static_passed_cargo_deferred"
            ),
            "{label} should mirror the script_absorption route-owner split status"
        );
    }
    assert!(
        DATE_MAP.contains("Runtime 15 M3 script-absorption route-owner split"),
        "date map should mirror the script_absorption route-owner split slice"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "script_absorption/legacy_crate.rs",
            "script_absorption/split_layout.rs",
            "runtime_15_script_absorption_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 plan",
        NUMBERED_STATUS_RECORDS,
        &[
            "frameworks_02_m3_script_absorption_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 script-absorption route-owner split",
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

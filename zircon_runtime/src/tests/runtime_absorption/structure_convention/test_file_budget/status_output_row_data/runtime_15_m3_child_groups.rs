use super::*;

#[test]
fn runtime_15_status_output_m3_row_data_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs",
    );
    let runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    let runtime_15_m3 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
    );
    let foundation_guards = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );
    let lock_poison_status = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );
    let module_convention_status = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs",
    );
    let review_status_sync = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync.rs",
    );
    let status_support = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let ui_tests_second = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
    );
    let production_guard_support = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
    );
    let expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
    );
    let expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "top-level status rows include every Runtime 15 M3 child group",
        &parent,
        &[
            "runtime_15::RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_UI_TESTS_FIRST_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_SCENE_SCRIPT_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_UI_TESTS_SECOND_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 status row parent exposes M3 child groups",
        &runtime_15,
        &[
            "pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 status row parent is a child-group aggregator",
        &runtime_15_m3,
        &[
            "#[path = \"m3/foundation_guards.rs\"]",
            "#[path = \"m3/lock_poison_status.rs\"]",
            "#[path = \"m3/module_convention_status.rs\"]",
            "#[path = \"m3/review_status_sync.rs\"]",
            "#[path = \"m3/status_support.rs\"]",
            "#[path = \"m3/ui_tests_second.rs\"]",
            "#[path = \"m3/production_guard_support.rs\"]",
            "pub(super) const FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 graphics dead-code guard module split",
        "Runtime 15 M3 UI runtime input ownership test folder split",
        "Runtime 15 M3 status output Runtime 15 M3 row data split",
        "Runtime 15 M3 production file budget guard child-owner split",
    ] {
        assert!(
            !runtime_15_m3.contains(moved_row),
            "expected_status_row_data/runtime_15/m3.rs should delegate row literals instead of keeping {moved_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 M3 child row owners keep representative row literals",
        &(foundation_guards.clone()
            + lock_poison_status.as_str()
            + module_convention_status.as_str()
            + review_status_sync.as_str()
            + status_support.as_str()
            + ui_tests_second.as_str()
            + production_guard_support.as_str()),
        &[
            "Runtime 15 M3 graphics dead-code guard module split",
            "Runtime 15 M3 production direct lock unwrap global gate",
            "Runtime 15 M3 lock-poison status row-data child-owner split",
            "Runtime 15 M3 module convention gate output contract",
            "Runtime 15 M3 module-convention status row-data child-owner split",
            "Runtime 15 M3 status output Runtime 15 M3 row data split",
            "Runtime 15 M3 UI runtime input ownership test folder split",
            "Runtime 15 M3 F19 scene renderer construction top-row closed status sync",
            "Runtime 15 M3 status output M3 row data child-owner split",
            "Runtime 15 M3 review top-row status row-data child-owner split",
            "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    for moved_lock_poison_row in [
        "Runtime 15 M3 lock poison policy guard folder split",
        "Runtime 15 M3 core runtime lock poison guard child-owner split",
        "Runtime 15 M3 F2 lock poison recovery guard",
        "Runtime 15 M3 production direct lock unwrap global gate",
        "Runtime 15 M3 config store lock poison recovery",
        "Runtime 15 M3 core runtime devtools lock poison recovery",
        "Runtime 15 M3 core handle diagnostics lock poison recovery",
        "Runtime 15 M3 core handle time lock poison recovery",
        "Runtime 15 M3 core handle states lock poison recovery",
        "Runtime 15 M3 core runtime task lock poison recovery",
        "Runtime 15 M3 core runtime profiling lock poison recovery",
        "Runtime 15 M3 core handle registry lock poison recovery",
        "Runtime 15 M3 plugin bridge table lock poison recovery",
        "Runtime 15 M3 native live-host bridge methods lock poison recovery",
        "Runtime 15 M3 navigation lock poison recovery",
        "Runtime 15 M3 dynamic API session lock poison recovery",
        "Runtime 15 M3 dynamic scene spawn task lock poison recovery",
        "Runtime 15 M3 scene ECS parallel executor lock poison recovery",
        "Runtime 15 M3 core resource manager lock poison recovery",
        "Runtime 15 M3 asset project manager lock poison recovery",
        "Runtime 15 M3 asset worker pool lock poison recovery",
        "Runtime 15 M3 WGPU render framework lock poison recovery",
        "Runtime 15 M3 RHI WGPU render device lock poison recovery",
        "Runtime 15 M3 animation manager lock poison recovery",
        "Runtime 15 M3 input runtime manager lock poison recovery",
        "Runtime 15 M3 script VM registry lock poison recovery",
        "Runtime 15 M3 ZrVM real backend runtime lock poison recovery",
        "Runtime 15 M3 VM plugin manager selected-backend lock poison recovery",
    ] {
        assert!(
            !foundation_guards.contains(moved_lock_poison_row),
            "foundation_guards.rs should delegate lock-poison status rows to lock_poison_status.rs instead of keeping {moved_lock_poison_row}"
        );
        assert!(
            lock_poison_status.contains(moved_lock_poison_row),
            "lock_poison_status.rs should own moved lock-poison status row {moved_lock_poison_row}"
        );
    }
    for moved_module_convention_row in [
        "Runtime 15 M3 module convention gate output contract",
        "Runtime 15 M3 module convention non-render debt guard",
        "Runtime 15 M3 render-scoped migration debt handoff gate",
        "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup",
        "Runtime 15 M3 module convention gate audit-clear status mirror",
        "Runtime 15 M3 module convention audit script family naming cleanup",
    ] {
        assert!(
            !foundation_guards.contains(moved_module_convention_row),
            "foundation_guards.rs should delegate module-convention status rows to module_convention_status.rs instead of keeping {moved_module_convention_row}"
        );
        assert!(
            module_convention_status.contains(moved_module_convention_row),
            "module_convention_status.rs should own moved module-convention status row {moved_module_convention_row}"
        );
    }
    for moved_top_row in [
        "Runtime 15 M3 D13 importer top-row closed status sync",
        "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync",
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync",
        "Runtime 15 M3 D7 core workspace dependency top-row closed status sync",
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync",
        "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync",
        "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync",
        "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync",
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync",
    ] {
        assert!(
            !foundation_guards.contains(moved_top_row),
            "foundation_guards.rs should delegate review top-row status rows to review_status_sync.rs instead of keeping {moved_top_row}"
        );
        assert!(
            review_status_sync.contains(moved_top_row),
            "review_status_sync.rs should own moved review top-row status row {moved_top_row}"
        );
    }

    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_status_row_data.rs",
            parent.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
            runtime_15_m3.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
            foundation_guards.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
            lock_poison_status.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs",
            module_convention_status.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync.rs",
            review_status_sync.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
            status_support.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
            ui_tests_second.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
            production_guard_support.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 expected status map records M3 child-owner split",
        &expected_status_map,
        &[
            "Runtime 15 M3 status output M3 row data child-owner split",
            "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 lock-poison status row-data child-owner split",
            "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 module-convention status row-data child-owner split",
            "runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 review top-row status row-data child-owner split",
            "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected date map records M3 child-owner split",
        &expected_date_map,
        &[
            "Runtime 15 M3 status output M3 row data child-owner split",
            "2026-06-24",
            "Runtime 15 M3 lock-poison status row-data child-owner split",
            "Runtime 15 M3 module-convention status row-data child-owner split",
            "Runtime 15 M3 review top-row status row-data child-owner split",
            "2026-06-28",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 production support row data",
            production_guard_support.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status output M3 row data child-owner split",
                "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
                "runtime_15_status_output_m3_row_data_child_owner_split",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 lock-poison row data",
            lock_poison_status.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 lock-poison status row-data child-owner split",
                "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
                "runtime_15_status_output_m3_row_data_child_owner_split",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 module-convention row data",
            module_convention_status.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 module-convention status row-data child-owner split",
                "runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs",
                "runtime_15_status_output_m3_row_data_child_owner_split",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 review status-sync row data",
            review_status_sync.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 review top-row status row-data child-owner split",
                "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync.rs",
                "runtime_15_status_output_m3_row_data_child_owner_split",
            ],
        );
    }
}

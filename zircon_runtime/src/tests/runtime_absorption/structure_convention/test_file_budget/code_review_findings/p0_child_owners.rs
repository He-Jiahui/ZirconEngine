use super::super::*;

#[path = "p0_child_owners/budgets.rs"]
mod budgets;
#[path = "p0_child_owners/delegation.rs"]
mod delegation;
#[path = "p0_child_owners/route_ownership.rs"]
mod route_ownership;
#[path = "p0_child_owners/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STRUCTURE_GUARD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners.rs";
pub(super) const SLICE: &str = "Runtime 15 M3 P0 robustness review guard child-owner split";
pub(super) const STATUS: &str =
    "runtime_15_p0_robustness_review_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const DATE: &str = "2026-06-30";
pub(super) const GUARD: &str = "runtime_15_p0_robustness_review_guards_are_child_owners";
pub(super) const FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 P0 robustness structure guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS: &str =
    "runtime_15_p0_robustness_structure_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_DATE: &str = "2026-07-03";
pub(super) const FOLDER_BACKED_GUARD: &str =
    "runtime_15_p0_robustness_structure_guard_is_folder_backed";
pub(super) const FOLDER_BACKED_STATUS_GUARD: &str =
    "runtime_15_p0_robustness_structure_guard_folder_backed_status_is_current";
pub(super) const BUDGET_GUARD: &str =
    "runtime_15_p0_robustness_structure_guard_budgets_are_focused";

pub(super) const PARENT: &str = "tests/runtime_absorption/code_review_findings/p0_robustness.rs";
pub(super) const NATIVE_HOST_CALLBACKS: &str =
    "tests/runtime_absorption/code_review_findings/p0_robustness/native_host_callbacks.rs";
pub(super) const LOCK_POISON: &str =
    "tests/runtime_absorption/code_review_findings/p0_robustness/lock_poison.rs";
pub(super) const RENDER_SUBMIT: &str =
    "tests/runtime_absorption/code_review_findings/p0_robustness/render_submit.rs";
pub(super) const NATIVE_FIXTURE: &str =
    "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs";
pub(super) const NATIVE_FIXTURE_SDK_MACRO: &str =
    "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs";
pub(super) const NATIVE_FIXTURE_IMPORTER: &str =
    "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/importer_manifest.rs";
pub(super) const PRIORITY_RECOMMENDATION: &str =
    "tests/runtime_absorption/code_review_findings/p0_robustness/priority_recommendation.rs";
pub(super) const STRUCTURE_GUARD_ROWS: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_robustness.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const REVIEW_GUARDS: &[&str] = &[
    "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
    "review_f2_scene_eventbus_locks_recover_after_poison",
    "review_f4_render_submit_capability_gaps_return_typed_errors",
    "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
    "review_d13_native_fixture_importer_is_manifest_described",
    "review_priority_recommendation_tracks_current_remaining_work",
];

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/delegation.rs",
        FOLDER_BACKED_GUARD,
    ),
    (
        "route_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/route_ownership.rs",
        GUARD,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/status_mirrors.rs",
        FOLDER_BACKED_STATUS_GUARD,
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/budgets.rs",
        BUDGET_GUARD,
    ),
];

pub(super) struct P0RobustnessSources {
    pub(super) parent: String,
    pub(super) native_host_callbacks: String,
    pub(super) lock_poison: String,
    pub(super) render_submit: String,
    pub(super) native_fixture: String,
    pub(super) native_fixture_sdk_macro: String,
    pub(super) native_fixture_importer: String,
    pub(super) priority_recommendation: String,
}

impl P0RobustnessSources {
    pub(super) fn all_sources(&self) -> [(&'static str, &str); 8] {
        [
            (PARENT, self.parent.as_str()),
            (NATIVE_HOST_CALLBACKS, self.native_host_callbacks.as_str()),
            (LOCK_POISON, self.lock_poison.as_str()),
            (RENDER_SUBMIT, self.render_submit.as_str()),
            (NATIVE_FIXTURE, self.native_fixture.as_str()),
            (
                NATIVE_FIXTURE_SDK_MACRO,
                self.native_fixture_sdk_macro.as_str(),
            ),
            (
                NATIVE_FIXTURE_IMPORTER,
                self.native_fixture_importer.as_str(),
            ),
            (
                PRIORITY_RECOMMENDATION,
                self.priority_recommendation.as_str(),
            ),
        ]
    }
}

pub(super) fn read_p0_robustness_sources() -> P0RobustnessSources {
    P0RobustnessSources {
        parent: read_runtime_src(PARENT),
        native_host_callbacks: read_runtime_src(NATIVE_HOST_CALLBACKS),
        lock_poison: read_runtime_src(LOCK_POISON),
        render_submit: read_runtime_src(RENDER_SUBMIT),
        native_fixture: read_runtime_src(NATIVE_FIXTURE),
        native_fixture_sdk_macro: read_runtime_src(NATIVE_FIXTURE_SDK_MACRO),
        native_fixture_importer: read_runtime_src(NATIVE_FIXTURE_IMPORTER),
        priority_recommendation: read_runtime_src(PRIORITY_RECOMMENDATION),
    }
}

pub(super) fn folder_backed_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        blob.push_str(&child_source);
        blob.push('\n');
    }
    blob
}

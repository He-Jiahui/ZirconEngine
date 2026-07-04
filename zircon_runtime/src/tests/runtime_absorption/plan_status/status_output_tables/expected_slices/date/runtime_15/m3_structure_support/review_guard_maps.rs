#[path = "review_guard_maps/typed_error_maps.rs"]
mod typed_error_maps;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = typed_error_maps::expected_date_for_slice(slice) {
        return Some(date);
    }

    match slice {
        "Runtime 15 M3 code review findings test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 code-review standalone harness current-path sync" => Some("2026-07-03"),
        "Runtime 15 M3 P0 robustness review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 P0 robustness structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 P0 robustness root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 P0 route ownership guard child split" => Some("2026-07-05"),
        "Runtime 15 M3 P0 native fixture review guard leaf-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 F8 API convergence review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 F8 child-owner structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 F8 child-owner root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 F8 route ownership guard child split" => Some("2026-07-05"),
        "Runtime 15 M3 F8 descriptor review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 late API cleanup review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 late API cleanup structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 late API cleanup root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 code review findings structure guard child-owner split" => Some("2026-06-29"),
        "Runtime 15 M3 code review findings structure guard children folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings structure guard children budget-status child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings structure guard children root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 structure guard plugin-importer child split" => Some("2026-07-05"),
        "Runtime 15 M3 P0 native fixture leaf-owner guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 P0 native fixture leaf-owner root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings status-doc guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 code review findings status-doc guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings status-doc status-mirror child-owner split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings status-doc source anchors child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings status-doc source anchors folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings status-doc status anchors child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 code review findings status-doc status anchors folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings status-doc child-anchor list child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 code review findings status-doc root inventory child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 code review findings status-doc status anchor guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings status-doc status-anchor child-ownership child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 code review findings folder-backed summary child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings folder-backed summary guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings folder-backed summary child-ownership guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings source inventory child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings source inventory folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings source inventory status-mirror child-owner split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings direct assertions guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings direct assertions child-ownership guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings F12 direct assertions child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 code review findings F12 direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings render direct assertions child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 code review findings render direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings F8 direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings F8 direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings P0 direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings P0 direct assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings structure guard folder-backed summary child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings structure guard folder-backed summary guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings structure guard typed-error child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings structure guard typed-error folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 code review findings typed-error structure guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 typed-error structure guard root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error child-ownership guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error child-ownership root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error structure assertions guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 typed-error structure assertions guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 typed-error convergence mounts guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error convergence mounts root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error native plugin loader structure guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error native plugin loader routes child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error native plugin loader routes source helper child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error native plugin loader source helper child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error structure moved-guard absence child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 typed-error moved-guard absence root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 plugin-importer DX structure guard root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX status-doc guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer DX status-doc root inventory child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX source inventory guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer DX structure assertions guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX structure assertions guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 plugin-importer DX review mounts guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review top-row status row-data child-owner split" => Some("2026-06-28"),
        "Runtime 15 M3 D-S7 static plugin manifest generation/parity review sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D7 core workspace dependency top-row closed status sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D7 core workspace dependency inheritance guard" => Some("2026-06-28"),
        "Runtime 15 M3 D8 runtime registration builder original evidence paths" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync" => Some("2026-06-28"),
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D9 editor/runtime mirror consumer guard" => Some("2026-06-28"),
        "Runtime 15 M3 D5 editor authoring macro consumer guard" => Some("2026-06-28"),
        "Runtime 15 M3 D12 runtime helper export macro review sync" => Some("2026-06-28"),
        "Runtime 15 M3 D1 capability single-source review sync" => Some("2026-06-28"),
        "Runtime 15 M3 D10 animation/physics bridge call migration" => Some("2026-06-28"),
        "Runtime 15 M3 D11 animation/physics TestRuntime fixture migration" => {
            Some("2026-06-28")
        }
        "Runtime 15 M3 D13 importer manifest parity guard" => Some("2026-06-28"),
        "Runtime 15 M3 P0/DX priority D13 parity sync" => Some("2026-06-28"),
        "Runtime 15 M3 D13 importer top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync" => Some("2026-06-28"),
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync" => Some("2026-06-28"),
        _ => None,
    }
}

// Runtime 15 M3 code-review row-data owner child split anchor mirror:
// runtime_15_code_review_rows_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs
// runtime_15_code_review_rows_row_data_owner_is_child_backed
// Runtime 15 M3 plugin-importer row-data owner child split anchor mirror:
// runtime_15_plugin_importer_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/review_guards.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/status_docs.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/source_inventory.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/structure_assertions.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/row_data_owner.rs
// runtime_15_plugin_importer_rows_row_data_owner_is_child_backed

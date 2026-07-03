pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 code-review standalone harness current-path sync" => Some("2026-07-03"),
        "Runtime 15 M3 P0 robustness review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 P0 robustness structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 P0 native fixture review guard leaf-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 F8 API convergence review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 F8 child-owner structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 F8 descriptor review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 late API cleanup review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 late API cleanup structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 code review findings structure guard child-owner split" => Some("2026-06-29"),
        "Runtime 15 M3 code review findings structure guard children folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 P0 native fixture leaf-owner guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 code review findings status-doc guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 code review findings status-doc guard folder-backed split" => {
            Some("2026-07-02")
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
        "Runtime 15 M3 code review findings folder-backed summary child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings folder-backed summary guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings source inventory child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings source inventory folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings direct assertions guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code review findings F12 direct assertions child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings render direct assertions child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 code review findings F8 direct assertions child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 code review findings P0 direct assertions child-owner split" => {
            Some("2026-06-30")
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
        "Runtime 15 M3 typed-error structure assertions guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 typed-error structure assertions guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error structure moved-guard absence child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX structure guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX structure assertions guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 plugin-importer DX structure assertions guard folder-backed split" => {
            Some("2026-07-02")
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
        "Runtime 15 M3 typed-error convergence guard child-owner split" => Some("2026-06-25"),
        "Runtime 15 M3 native plugin loader typed-error review guard child-owner split" => {
            Some("2026-06-29")
        }
        "Runtime 15 M3 native ABI surfaces typed-error review guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 native plugin descriptor ABI typed-error review guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 UI input typed-error review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 review guard status row-data child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 review guard row-data topic child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 review-guard typed-error row-data child split" => Some("2026-07-04"),
        "Runtime 15 M3 code-review row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 typed-error structure row-data child split" => Some("2026-07-03"),
        "Runtime 15 M3 code-review structure-guard row-data folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 code-review structure-guard root-and-children row-data child split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 code review findings status-row source child-tree sync" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 typed-error structure status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error structure status-doc guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 typed-error source inventory guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 native manifest sources typed-error review guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 script host typed-error review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 scene world typed-error review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 asset loader typed-error review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 asset records typed-error review guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 shader prewarm CLI typed-error review guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 native live-host typed-error review guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 native live-host lifecycle-paths typed-error review guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split" => {
            Some("2026-06-30")
        }
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

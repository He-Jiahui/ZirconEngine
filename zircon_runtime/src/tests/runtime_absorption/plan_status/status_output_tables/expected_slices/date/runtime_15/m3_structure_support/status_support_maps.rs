#[path = "status_support_maps/plan_doc_support_maps.rs"]
mod plan_doc_support_maps;
#[path = "status_support_maps/row_data_maps.rs"]
mod row_data_maps;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = row_data_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = plan_doc_support_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    match slice {
        "Runtime 15 M3 test file budget root-layout child split" => Some("2026-06-23"),
        "Runtime 15 M3 status output Runtime 15 row data split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 15 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 Runtime 15 row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 Runtime 15 row-data row-ownership child split" => Some("2026-07-04"),
        "Runtime 15 M3 Runtime 15 row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 status output Runtime 15 foundation row data split" => Some("2026-06-27"),
        "Runtime 15 M3 foundation row-data topic child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 foundation row-data 73-row docs sync" => Some("2026-07-01"),
        "Runtime 15 M3 foundation row-data stale-count prose guard" => Some("2026-07-03"),
        "Runtime 15 M3 foundation row-data priority-doc frontmatter sync" => Some("2026-07-03"),
        "Runtime 15 M3 status output Runtime 15 M2 row data split" => Some("2026-06-29"),
        "Runtime 15 M3 M2 row-data guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 M2 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 M2 row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 M2 row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 support Hub project-actions tests child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 support Hub runtime-state tests child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 production file budget core runtime guard split" => Some("2026-06-23"),
        "Runtime 15 M3 render shader template assembly guard support child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 render shader template assembly assertion contract child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 shader prewarm manifest guard child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 status output Runtime 15 M4 row data split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 15 M4 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 Runtime 15 M4 row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 Runtime 15 M4 row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 status output expected-slice maps split" => Some("2026-06-23"),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 status output expected-slice guard maps child-owner split" => Some("2026-06-25"),
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 status output Runtime 15 M3 row data split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 15 M3 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 Runtime 15 M3 row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 Runtime 15 M3 row-data root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 test file budget root-layout status scan child split" => Some("2026-06-23"),
        "Runtime 15 M3 test file budget root-layout folder-backed guard child split" => {
            Some("2026-06-24")
        }
        "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 root-layout status-output Runtime 15 row-data child-source sync" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 test file budget parent guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 historical oversized test roots closeout" => Some("2026-06-23"),
        "Runtime 15 M3 asset test-budget guard child-owner split" => Some("2026-06-23"),
        "Runtime 15 M3 UI asset test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI asset surface index test folder split" => Some("2026-06-24"),
        "Runtime 15 M3 UI asset MUI web form style test folder split" => Some("2026-06-24"),
        "Runtime 15 M3 UI asset MUI X web style test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI asset MUI web style test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI taffy layout pass test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI runtime window input pump test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI runtime window event ABI child folder split" => Some("2026-06-23"),
        "Runtime 15 M3 test file budget root-layout UI child split" => Some("2026-06-23"),
        "Runtime 15 M3 UI widget text input keyboard test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI focus navigation test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI runtime input manager test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI runtime input ownership test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 production file budget guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output variable evidence anchors" => Some("2026-06-24"),
        "Runtime 15 M3 status output evidence anchors guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 evidence anchors status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 evidence anchors root inventory child split" => Some("2026-07-04"),
        _ => None,
    }
}

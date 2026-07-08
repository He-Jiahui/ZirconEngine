pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 test file budget root-layout status scan child split" => Some("2026-06-23"),
        "Runtime 15 M3 test file budget root-layout folder-backed guard child split" => {
            Some("2026-06-24")
        }
        "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 test file budget root-layout assertions guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 root-layout status-output Runtime 15 row-data child-source sync" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 root entries/root-layout current-child route sync" => Some("2026-07-08"),
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
        _ => None,
    }
}

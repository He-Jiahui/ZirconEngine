pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review guard status row-data child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 review guard row-data topic child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 review-guard typed-error row-data child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard status-support row-data folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 review-guard status-support folder-backed guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard status-support folder-backed split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard status-support anchor mirror cleanup" => Some("2026-07-06"),
        "Runtime 15 M3 review-guard status-support anchor-mirror cleanup guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard status-support rows guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard status-support rows split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard typed-error rows guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 review-guard typed-error rows split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 code-review row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 review-guard rows row-data owner child split" => Some("2026-07-07"),
        "Runtime 15 M3 review-guard status-support review rows row-data owner child split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 review-guard status-support review rows guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 plugin-importer row-data owner child split" => Some("2026-07-04"),
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
        "Runtime 15 M3 code review findings split row/map source sync" => Some("2026-07-07"),
        _ => None,
    }
}

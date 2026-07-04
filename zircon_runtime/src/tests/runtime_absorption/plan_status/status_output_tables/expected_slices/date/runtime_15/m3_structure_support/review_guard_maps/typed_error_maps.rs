pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review guard typed-error expected-slice map child split" => {
            Some("2026-07-05")
        }
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
        "Runtime 15 M3 typed-error structure status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 typed-error structure status-doc guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 typed-error status-doc doc mirrors folder-backed split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 typed-error status-doc source helper child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 typed-error source inventory guard folder-backed split" => Some("2026-07-04"),
        "Runtime 15 M3 typed-error source inventory source helper child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error source inventory child sources folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory child sources structure guard child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory child inventory folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory child inventory status-current child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory metadata child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error source inventory metadata status-current child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory delegation child split" => Some("2026-07-05"),
        "Runtime 15 M3 typed-error source inventory delegation folder-backed child split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 typed-error source inventory delegation folder-backed ownership child split" => {
            Some("2026-07-05")
        }
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
        _ => None,
    }
}

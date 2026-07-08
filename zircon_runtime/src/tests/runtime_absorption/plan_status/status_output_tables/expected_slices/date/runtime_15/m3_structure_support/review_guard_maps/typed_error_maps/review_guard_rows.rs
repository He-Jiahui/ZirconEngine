pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
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

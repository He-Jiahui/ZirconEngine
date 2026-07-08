pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 production file budget core runtime guard split" => Some("2026-06-23"),
        "Runtime 15 M3 render shader template assembly guard support child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 render shader template assembly assertion contract child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 mesh pipeline shader source tests child-owner split" => Some("2026-07-05"),
        "Runtime 15 M3 shader prewarm manifest guard child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 shader prewarm manifest current-child route sync" => Some("2026-07-08"),
        _ => None,
    }
}

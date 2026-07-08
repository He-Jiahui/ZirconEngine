pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 Hybrid GI extract scene-source naming hard cutover" => Some("2026-06-27"),
        "Runtime 15 M2 DDS upload policy naming hard cutover" => Some("2026-06-27"),
        _ => None,
    }
}

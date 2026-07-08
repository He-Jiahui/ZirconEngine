pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 material asset schema-v1 defaults naming hard cutover" => Some("2026-06-27"),
        "Runtime 15 M2 font/UI asset schema naming hard cutover" => Some("2026-06-29"),
        "Runtime 15 M2 font render-mode priority fixture naming hard cutover" => Some("2026-06-29"),
        _ => None,
    }
}

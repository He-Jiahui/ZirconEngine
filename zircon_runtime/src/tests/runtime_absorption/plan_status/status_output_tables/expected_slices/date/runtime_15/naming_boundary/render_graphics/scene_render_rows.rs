pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 scene dynamic document v1 owner naming hard cutover" => Some("2026-06-27"),
        "Runtime 15 M2 scene render layer schema-v1 mask naming hard cutover" => Some("2026-06-27"),
        "Runtime 15 M2 render layer schema-v1 mask API naming hard cutover" => Some("2026-06-27"),
        _ => None,
    }
}

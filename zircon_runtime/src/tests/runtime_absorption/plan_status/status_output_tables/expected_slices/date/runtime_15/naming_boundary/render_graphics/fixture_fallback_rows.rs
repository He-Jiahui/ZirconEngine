pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 render feature fallback capability naming hard cutover" => {
            Some("2026-06-27")
        }
        "Runtime 15 M2 render material stale texture fixture naming hard cutover" => {
            Some("2026-06-27")
        }
        "Runtime 15 M2 render graph fallback fixture naming hard cutover" => Some("2026-06-27"),
        _ => None,
    }
}

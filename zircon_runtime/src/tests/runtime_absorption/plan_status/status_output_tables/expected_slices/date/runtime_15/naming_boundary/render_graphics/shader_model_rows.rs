pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 render shader definition bare-flag naming hard cutover" => {
            Some("2026-06-27")
        }
        "Runtime 15 M2 GPU model embedded primitive naming hard cutover" => Some("2026-06-27"),
        "Runtime 15 M2 frame extract snapshot adapter naming hard cutover" => Some("2026-06-27"),
        "Runtime 15 M2 core framework render fixture naming hard cutover" => Some("2026-06-27"),
        _ => None,
    }
}

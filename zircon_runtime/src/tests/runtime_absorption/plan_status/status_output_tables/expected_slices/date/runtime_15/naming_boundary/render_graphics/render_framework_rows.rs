pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 graphics render-framework receiver naming hard cutover" => {
            Some("2026-06-25")
        }
        "Runtime 15 M2 render framework trait/construction owner naming hard cutover" => {
            Some("2026-06-27")
        }
        "Runtime 15 M2 graphics construction new owner naming hard cutover" => Some("2026-06-27"),
        _ => None,
    }
}

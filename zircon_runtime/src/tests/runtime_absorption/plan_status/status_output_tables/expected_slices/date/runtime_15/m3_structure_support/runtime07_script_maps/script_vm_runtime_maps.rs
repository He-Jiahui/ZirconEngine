pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 script VM test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 script VM primary guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 script VM hot-reload coordinator test folder split" => Some("2026-06-24"),
        "Runtime 15 M3 script VM hot-reload guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 native live-host tests folder split" => Some("2026-06-24"),
        "Runtime 15 M3 native plugin loader real fixture test folder split" => Some("2026-06-24"),
        "Runtime 15 M3 extension registry bridge test folder split" => Some("2026-06-24"),
        "Runtime 15 M3 manifest contributions test folder split" => Some("2026-06-24"),
        "Runtime 15 M3 manifest contributions runtime-family test child-owner split" => {
            Some("2026-07-01")
        }
        _ => None,
    }
}

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs listing prose full inventory sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs moved mirror full inventory sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs root inventory child split" => Some("2026-07-04"),
        _ => None,
    }
}

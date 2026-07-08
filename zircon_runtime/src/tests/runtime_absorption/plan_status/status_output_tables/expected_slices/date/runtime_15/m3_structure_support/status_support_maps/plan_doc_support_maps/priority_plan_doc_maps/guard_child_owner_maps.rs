pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs guard child-owner split" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs child prose full inventory sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs guard-test child-owner split" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs guard-test child prose full inventory sync" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 priority plan docs moved guard path mirror" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs guard inventory row-data source sync" => {
            Some("2026-07-04")
        }
        _ => None,
    }
}

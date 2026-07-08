pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard typed-error structure maps folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error structure row-data owner child split" => Some("2026-07-07"),
        _ => None,
    }
}

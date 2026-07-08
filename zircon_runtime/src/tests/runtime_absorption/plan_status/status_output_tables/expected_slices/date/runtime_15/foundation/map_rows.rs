const SLICE: &str = "Runtime 15 M3 foundation expected-slice maps folder-backed split";

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == SLICE {
        Some("2026-07-07")
    } else {
        None
    }
}

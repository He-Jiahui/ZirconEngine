pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 runtime plugin package manifest test folder split" {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M3 runtime plugin package manifest capability-status test child-owner split"
    {
        Some("2026-07-01")
    } else if slice
        == "Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split"
    {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split" {
        Some("2026-07-03")
    } else if slice
        == "Runtime 15 M3 runtime plugin lifecycle fixture row-data current-child route sync"
    {
        Some("2026-07-08")
    } else if slice == "Runtime 15 M3 export build plan test folder split" {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M3 export build plan profile feature matrix test child-owner split"
    {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 export build plan platform test folder split" {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M3 export build plan platform release-adapter test child-owner split"
    {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M3 gameplay host test folder split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M3 script VM gameplay host guard child-owner split" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M3 shader prewarm manifest test folder split" {
        Some("2026-06-23")
    } else {
        None
    }
}

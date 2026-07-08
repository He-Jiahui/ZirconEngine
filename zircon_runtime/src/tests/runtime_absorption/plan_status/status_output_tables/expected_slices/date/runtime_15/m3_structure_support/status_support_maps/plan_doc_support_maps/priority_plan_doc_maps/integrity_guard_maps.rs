pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs code-path integrity guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs test-path integrity guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs frontmatter status guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs frontmatter uniqueness guard" => Some("2026-07-03"),
        "Runtime 15 M3 priority plan docs required header sections guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs plan-source cross-link guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs guard-test listing guard" => Some("2026-07-01"),
        _ => None,
    }
}

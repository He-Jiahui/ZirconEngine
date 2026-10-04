#[test]
fn menu_chrome_expansion_reserves_raw_and_slot_upper_bound() {
    let source = include_str!("../menu_chrome.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("menu chrome implementation before tests");

    assert!(implementation.contains("let slot_count = menus.row_count().max(MENU_SLOT_COUNT);"));
    assert!(implementation.contains(
        "let mut output_nodes = Vec::with_capacity(raw_nodes.len().saturating_add(slot_count));"
    ));
}

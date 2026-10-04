#[test]
fn specialized_options_are_projected_once_per_node() {
    let source = include_str!("../mod.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");

    assert!(implementation.contains("projected_command_palette_option_rows"));
    assert!(implementation.contains("projected_notification_center_option_rows"));
    assert!(!implementation.contains("projected_command_palette_options("));
    assert!(!implementation.contains("projected_notification_center_options("));
}

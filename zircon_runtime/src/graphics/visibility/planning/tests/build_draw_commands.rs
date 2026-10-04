#[test]
fn draw_command_builder_preallocates_visible_instances() {
    let source = include_str!("../build_draw_commands.rs");
    let capacity = concat!("Vec::with_capacity(", "visible_instance_count)");

    assert!(source.contains(capacity));
}

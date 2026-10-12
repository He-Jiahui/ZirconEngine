#[test]
fn hot_update_moves_owned_candidates_into_the_single_plugin_load_report() {
    let source = include_str!("../hot_update_application.rs");
    let deep_clone = ["discovered: vec![candidate", ".clone()]"].concat();

    assert!(!source.contains(&deep_clone));
    assert!(source.contains("NativePluginLoadReport::from_discovered(vec![candidate])"));
}

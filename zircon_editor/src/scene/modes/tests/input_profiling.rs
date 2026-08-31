#[test]
fn scene_mode_input_dispatch_profiles_checkpoint_copy_work() {
    let source = include_str!("../scene_mode_stack.rs");

    assert!(source.contains("profile_scope!(\"editor\", \"scene_mode\", \"input_dispatch\")"));
    for counter in [
        "scene_mode_input_dispatch_count",
        "scene_mode_input_checkpoint_count",
        "scene_mode_input_checkpoint_selection_item_count",
    ] {
        assert!(
            source.contains(&format!("\"{counter}\"")),
            "scene-mode input profiling must emit {counter}"
        );
    }
    assert!(
        source.contains("mode_checkpoint_selection_item_count"),
        "the input profile must measure the SelectionModel copy payload, not only mode calls"
    );
}

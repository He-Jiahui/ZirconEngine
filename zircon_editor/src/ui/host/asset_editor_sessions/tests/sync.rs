#[test]
fn syncing_instance_builds_one_reflection_model() {
    let source = include_str!("../sync.rs");
    let reflection_build = ["session.", "reflection_model()"].concat();

    assert_eq!(source.matches(&reflection_build).count(), 1);
}

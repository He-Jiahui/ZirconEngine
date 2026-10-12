#[test]
fn optimization_batch_fj_runtime466_mutable_generation_guard_uses_one_viewport_lookup() {
    let source = include_str!("../../framework/extract/viewport_generate_guard.rs");
    let mutable_guard = source
        .split("fn viewport_record_mut_after_generation_check_in")
        .nth(1)
        .expect("mutable generation guard source");
    let nested_validation = concat!("validate_viewport_generation(", "state, viewport, context");

    assert!(!mutable_guard.contains(nested_validation));
    assert_eq!(mutable_guard.matches(".get_mut(&viewport)").count(), 1);
}

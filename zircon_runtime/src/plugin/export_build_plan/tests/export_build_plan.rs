#[test]
fn fatal_presence_check_does_not_materialize_diagnostics() {
    let source = include_str!("../export_build_plan.rs");
    let allocating_check = ["!self.effective_fatal_diagnostics()", ".is_empty()"].concat();

    assert!(!source.contains(&allocating_check));
}

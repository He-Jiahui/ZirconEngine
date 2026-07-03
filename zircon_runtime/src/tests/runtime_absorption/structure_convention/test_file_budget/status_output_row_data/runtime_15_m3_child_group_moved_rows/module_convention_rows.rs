use super::*;

#[test]
fn runtime_15_m3_child_group_moved_module_convention_rows_are_child_owned() {
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_ROWS_PATH);
    let module_convention_status = read_runtime_src(MODULE_CONVENTION_STATUS_ROWS_PATH);

    for moved_module_convention_row in [
        "Runtime 15 M3 module convention gate output contract",
        "Runtime 15 M3 module convention non-render debt guard",
        "Runtime 15 M3 render-scoped migration debt handoff gate",
        "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup",
        "Runtime 15 M3 module convention gate audit-clear status mirror",
        "Runtime 15 M3 module convention audit script family naming cleanup",
    ] {
        assert!(
            !foundation_guards.contains(moved_module_convention_row),
            "foundation_guards.rs should delegate module-convention status rows to module_convention_status.rs instead of keeping {moved_module_convention_row}"
        );
        assert!(
            module_convention_status.contains(moved_module_convention_row),
            "module_convention_status.rs should own moved module-convention status row {moved_module_convention_row}"
        );
    }
}

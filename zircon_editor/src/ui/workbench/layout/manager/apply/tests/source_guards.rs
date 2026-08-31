#[test]
fn production_layout_commands_do_not_assume_validated_drawers() {
    let source = include_str!("../../apply.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production layout manager source");

    assert!(
        !production.contains(".expect("),
        "production layout commands must return typed errors instead of panicking"
    );
}

#[test]
fn apply_owners_stay_below_the_structure_budget() {
    const MAX_LINES: usize = 800;
    let owners = [
        ("apply.rs", include_str!("../../apply.rs")),
        ("apply/tests/mod.rs", include_str!("mod.rs")),
        ("apply/tests/no_op.rs", include_str!("no_op.rs")),
        (
            "apply/tests/drawer_commands.rs",
            include_str!("drawer_commands.rs"),
        ),
        (
            "apply/tests/source_guards.rs",
            include_str!("source_guards.rs"),
        ),
    ];

    for (path, source) in owners {
        let lines = source.lines().count();
        assert!(
            lines <= MAX_LINES,
            "{path} has {lines} lines and exceeds the {MAX_LINES}-line owner budget"
        );
    }
}

#[test]
fn native_materialization_indexes_package_export_rows_once() {
    let source = include_str!("../native.rs");
    let linear_lookup = [".find(|package| package.", "package_id == package_id)"].concat();
    let cloned_lookup = [".copied()", "\n            .cloned()"].concat();

    assert!(source.contains("native_dynamic_package_export_index(plan)"));
    assert!(source.contains("copy_native_dynamic_package_files"));
    assert!(!source.contains("fs::copy"));
    assert!(!source.contains(&linear_lookup));
    assert!(!source.contains(&cloned_lookup));
}

#[test]
fn native_package_reports_preallocate_plan_bounds() {
    let native_source = include_str!("../native.rs");
    let native_production = native_source
        .split("#[cfg(test)]")
        .next()
        .expect("native production source should remain available");
    let materialize_source = include_str!("../mod.rs");

    assert_eq!(
        native_production
            .matches("HashSet::with_capacity(plan.native_dynamic_packages.len())")
            .count(),
        2
    );
    assert_eq!(
        materialize_source
            .matches(".reserve(copied_package_capacity)")
            .count(),
        2
    );
}

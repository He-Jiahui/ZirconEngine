use super::registration_sources;

#[test]
fn registration_source_preserves_dependency_fast_paths() {
    let sources = registration_sources();
    let registration = sources.registration.as_str();

    assert!(registration.contains("fn dependency_names("));
    assert!(registration.contains("if dependencies.is_empty()"));
    assert!(registration.contains("return Arc::default();"));
    assert!(registration.contains("if let [dependency] = dependencies"));
    assert!(registration.contains("return Arc::<[RegistryName]>::from([dependency.name.clone()]);"));
    assert!(registration.contains("if let [first_dependency, second_dependency] = dependencies"));
    assert!(registration.contains("first_dependency.name.clone()"));
    assert!(registration.contains("second_dependency.name.clone()"));
    assert!(registration
        .contains("if let [first_dependency, second_dependency, third_dependency] = dependencies"));
    assert!(registration.contains("third_dependency.name.clone()"));
    assert!(registration.contains(
        "if let [first_dependency, second_dependency, third_dependency, fourth_dependency] = dependencies"
    ));
    assert!(registration.contains("fourth_dependency.name.clone()"));
    assert!(registration.contains("fifth_dependency.name.clone()"));
    assert!(registration.contains("Vec::with_capacity(dependencies.len())"));
    assert!(registration.contains("names.push(dependency.name.clone())"));
    assert!(registration.contains("dependencies: dependency_names(dependencies)"));
    let empty_dependency_fast_path_index = registration
        .find("if dependencies.is_empty()")
        .expect("dependency-name materialization should fast-path empty dependency slices");
    let single_dependency_arc_index = registration
        .find("if let [dependency] = dependencies")
        .expect("single dependency names should bypass the Vec-backed dependency list");
    let two_dependency_arc_index = registration
        .find("if let [first_dependency, second_dependency] = dependencies")
        .expect("two dependency names should bypass the Vec-backed dependency list");
    let three_dependency_arc_index = registration
        .find("if let [first_dependency, second_dependency, third_dependency] = dependencies")
        .expect("three dependency names should bypass the Vec-backed dependency list");
    let four_dependency_arc_index = registration
        .find("if let [first_dependency, second_dependency, third_dependency, fourth_dependency] = dependencies")
        .expect("four dependency names should bypass the Vec-backed dependency list");
    let five_dependency_arc_index = registration
        .find("fifth_dependency.name.clone()")
        .expect("five dependency names should bypass the Vec-backed dependency list");
    let dependency_vec_index = registration
        .find("let mut names = Vec::with_capacity(dependencies.len())")
        .expect("six-or-more dependency slices should keep pre-sized materialization");
    assert!(empty_dependency_fast_path_index < single_dependency_arc_index);
    assert!(single_dependency_arc_index < two_dependency_arc_index);
    assert!(two_dependency_arc_index < three_dependency_arc_index);
    assert!(three_dependency_arc_index < four_dependency_arc_index);
    assert!(four_dependency_arc_index < five_dependency_arc_index);
    assert!(five_dependency_arc_index < dependency_vec_index);
    let driver_dependency_validation_index = registration
        .find("fn validate_driver_dependencies(")
        .expect("driver dependency validation should have a dedicated helper");
    let driver_dependency_kind_helper_index = registration
        .find("fn validate_driver_dependency_kind(")
        .expect("driver dependency kind checking should be split from traversal");
    let driver_dependency_validation =
        &registration[driver_dependency_validation_index..driver_dependency_kind_helper_index];
    assert!(
        driver_dependency_validation
            .find("if dependencies.is_empty()")
            .unwrap()
            < driver_dependency_validation
                .find("if let [dependency] = dependencies")
                .unwrap()
    );
    assert!(driver_dependency_validation
        .contains("return validate_driver_dependency_kind(kind, name, second_dependency);"));
    assert!(driver_dependency_validation
        .contains("if let [first_dependency, second_dependency, third_dependency] = dependencies"));
    assert!(driver_dependency_validation
        .contains("return validate_driver_dependency_kind(kind, name, third_dependency);"));
    assert!(driver_dependency_validation.contains(
        "if let [first_dependency, second_dependency, third_dependency, fourth_dependency] ="
    ));
    assert!(driver_dependency_validation
        .contains("return validate_driver_dependency_kind(kind, name, fourth_dependency);"));
    assert!(driver_dependency_validation.contains("fifth_dependency"));
    assert!(driver_dependency_validation
        .contains("return validate_driver_dependency_kind(kind, name, fifth_dependency);"));
    let single_driver_dependency_index = driver_dependency_validation
        .find("if let [dependency] = dependencies")
        .expect("single driver dependency slices should validate directly");
    let two_driver_dependency_index = driver_dependency_validation
        .find("if let [first_dependency, second_dependency] = dependencies")
        .expect("two driver dependency slices should validate directly");
    let three_driver_dependency_index = driver_dependency_validation
        .find("if let [first_dependency, second_dependency, third_dependency] = dependencies")
        .expect("three driver dependency slices should validate directly");
    let four_driver_dependency_index = driver_dependency_validation
        .find("fourth_dependency")
        .expect("four driver dependency slices should validate directly");
    let five_driver_dependency_index = driver_dependency_validation
        .find("fifth_dependency")
        .expect("five driver dependency slices should validate directly");
    let driver_dependency_loop_index = driver_dependency_validation
        .find("for dependency in dependencies")
        .expect("six-or-more driver dependency slices should retain the loop");
    assert!(single_driver_dependency_index < two_driver_dependency_index);
    assert!(two_driver_dependency_index < three_driver_dependency_index);
    assert!(three_driver_dependency_index < four_driver_dependency_index);
    assert!(four_driver_dependency_index < five_driver_dependency_index);
    assert!(five_driver_dependency_index < driver_dependency_loop_index);
}

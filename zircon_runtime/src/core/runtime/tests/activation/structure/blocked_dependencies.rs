use super::fixture::{
    activation_source, activation_tests_source, blocked_dependencies_source, blocked_unload_source,
};

#[test]
fn blocked_dependency_matchers_stay_in_child_owners() {
    let activation_source = activation_source();
    let blocked_dependencies_source = blocked_dependencies_source();
    let blocked_unload_source = blocked_unload_source();
    let activation_tests_source = activation_tests_source();

    assert!(activation_source.contains("mod blocked_dependencies;"));
    assert!(activation_source.contains("mod blocked_unload;"));
    assert!(!activation_source.contains("fn first_blocked_unload("));
    assert!(!activation_source.contains("fn first_blocked_single_service_unload("));

    assert!(blocked_unload_source.contains("fn first_blocked_unload("));
    assert!(blocked_unload_source.contains("if let [service_name] = unload_order"));
    assert!(blocked_unload_source.contains("fn first_blocked_single_service_unload("));
    assert!(blocked_unload_source
        .contains("return first_blocked_single_service_unload(services, service_name);"));
    assert!(blocked_unload_source
        .contains("if let [first_service_name, second_service_name] = unload_order"));
    assert!(blocked_unload_source.contains("fn first_blocked_two_service_unload("));
    assert!(blocked_unload_source.contains("return first_blocked_two_service_unload("));
    assert!(blocked_unload_source.contains(
        "if let [first_service_name, second_service_name, third_service_name] = unload_order"
    ));
    assert!(blocked_unload_source.contains("fn first_blocked_three_service_unload("));
    assert!(blocked_unload_source.contains("return first_blocked_three_service_unload("));
    assert!(blocked_unload_source.contains("fn first_blocked_four_service_unload("));
    assert!(blocked_unload_source.contains("return first_blocked_four_service_unload("));
    assert!(blocked_unload_source.contains("fn first_blocked_five_service_unload("));
    assert!(blocked_unload_source.contains("return first_blocked_five_service_unload("));
    assert!(blocked_unload_source.contains("dependent_name == first_service_name"));
    assert!(blocked_unload_source.contains("dependent_name == second_service_name"));
    assert!(blocked_unload_source.contains("dependent_name == third_service_name"));
    assert!(blocked_unload_source.contains("dependent_name == fourth_service_name"));
    assert!(blocked_unload_source.contains("dependent_name == fifth_service_name"));
    assert!(!activation_source.contains("fn first_blocked_two_service_dependency("));
    assert!(!activation_source.contains("fn first_blocked_three_service_dependency("));
    assert!(!activation_source.contains("fn first_blocked_four_service_dependency("));
    assert!(!activation_source.contains("fn first_blocked_five_service_dependency("));
    assert!(!activation_source.contains("let mut second_service_blocked = false;"));
    assert!(!activation_source.contains("let mut fifth_service_blocked = false;"));
    assert!(!blocked_unload_source.contains("fn first_blocked_two_service_dependency("));
    assert!(!blocked_unload_source.contains("fn first_blocked_three_service_dependency("));
    assert!(!blocked_unload_source.contains("fn first_blocked_four_service_dependency("));
    assert!(!blocked_unload_source.contains("fn first_blocked_five_service_dependency("));

    assert!(blocked_dependencies_source.contains("enum TwoServiceDependencyMatch"));
    assert!(blocked_dependencies_source.contains("fn first_blocked_two_service_dependency("));
    assert!(blocked_dependencies_source.contains("enum ThreeServiceDependencyMatch"));
    assert!(blocked_dependencies_source.contains("fn first_blocked_three_service_dependency("));
    assert!(blocked_dependencies_source.contains("enum FourServiceDependencyMatch"));
    assert!(blocked_dependencies_source.contains("fn first_blocked_four_service_dependency("));
    assert!(blocked_dependencies_source.contains("enum FiveServiceDependencyMatch"));
    assert!(blocked_dependencies_source.contains("fn first_blocked_five_service_dependency("));
    assert!(blocked_dependencies_source.contains("let mut second_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("let mut third_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("let mut fourth_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("let mut fifth_service_blocked = false;"));
    assert!(blocked_dependencies_source.contains("if dependency == first_service_name"));
    assert!(blocked_dependencies_source.contains("if dependency == second_service_name"));
    assert!(blocked_dependencies_source.contains("dependency == third_service_name"));
    assert!(blocked_dependencies_source.contains("dependency == fourth_service_name"));
    assert!(blocked_dependencies_source.contains("dependency == fifth_service_name"));
    assert!(blocked_unload_source
        .contains("fn blocked_exact_service_result<const SERVICE_COUNT: usize>("));
    assert!(blocked_unload_source.contains("blocked_exact_service_result("));
    assert!(blocked_unload_source.contains("record_blocked_dependent(&mut blocked_index"));
    assert!(!blocked_unload_source.contains("first_blocked_dependents"));
    assert!(!blocked_unload_source.contains("second_blocked_dependents"));
    assert!(!blocked_unload_source.contains("third_blocked_dependents"));
    assert!(!blocked_unload_source.contains("fourth_blocked_dependents"));
    assert!(!blocked_unload_source.contains("fifth_blocked_dependents"));
    assert!(blocked_unload_source.contains("dependent_name == service_name"));

    let single_blocked_helper_index = blocked_unload_source
        .find("fn first_blocked_single_service_unload(")
        .expect("single-service blocked unload should stay in its own helper");
    let two_blocked_helper_index = blocked_unload_source
        .find("fn first_blocked_two_service_unload(")
        .expect("two-service blocked unload should stay in its own helper");
    let three_blocked_helper_index = blocked_unload_source
        .find("fn first_blocked_three_service_unload(")
        .expect("three-service blocked unload should stay in its own helper");
    let four_blocked_helper_index = blocked_unload_source
        .find("fn first_blocked_four_service_unload(")
        .expect("four-service blocked unload should stay in its own helper");
    let five_blocked_helper_index = blocked_unload_source
        .find("fn first_blocked_five_service_unload(")
        .expect("five-service blocked unload should stay in its own helper");
    assert!(single_blocked_helper_index < two_blocked_helper_index);
    assert!(two_blocked_helper_index < three_blocked_helper_index);
    assert!(three_blocked_helper_index < four_blocked_helper_index);
    assert!(four_blocked_helper_index < five_blocked_helper_index);

    let dependency_matcher_index = blocked_dependencies_source
        .find("fn dependency_slice_contains_service(")
        .expect("single-service blocked unload should route dependency matching through a helper");
    let two_dependency_matcher_index = blocked_dependencies_source
        .find("fn first_blocked_two_service_dependency(")
        .expect("two-service blocked unload should route dependency matching through a helper");
    let three_dependency_matcher_index = blocked_dependencies_source
        .find("fn first_blocked_three_service_dependency(")
        .expect("three-service blocked unload should route dependency matching through a helper");
    let four_dependency_matcher_index = blocked_dependencies_source
        .find("fn first_blocked_four_service_dependency(")
        .expect("four-service blocked unload should route dependency matching through a helper");
    let five_dependency_matcher_index = blocked_dependencies_source
        .find("fn first_blocked_five_service_dependency(")
        .expect("five-service blocked unload should route dependency matching through a helper");
    assert!(dependency_matcher_index < two_dependency_matcher_index);
    assert!(two_dependency_matcher_index < three_dependency_matcher_index);
    assert!(three_dependency_matcher_index < four_dependency_matcher_index);
    assert!(four_dependency_matcher_index < five_dependency_matcher_index);

    let single_blocked_helper_source =
        &blocked_unload_source[single_blocked_helper_index..two_blocked_helper_index];
    assert!(single_blocked_helper_source
        .contains("dependency_slice_contains_service(entry.dependencies.as_ref(), service_name)"));
    assert!(!single_blocked_helper_source.contains(".any(|dependency| dependency == service_name)"));

    let dependency_matcher_source =
        &blocked_dependencies_source[dependency_matcher_index..two_dependency_matcher_index];
    assert!(dependency_matcher_source.contains("match dependencies"));
    assert!(dependency_matcher_source.contains("[] => false"));
    assert!(dependency_matcher_source.contains("[dependency] => dependency == service_name"));
    assert!(dependency_matcher_source.contains("[first_dependency, second_dependency]"));
    assert!(dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency]"));
    assert!(dependency_matcher_source
        .contains("[first_dependency, second_dependency, third_dependency, fourth_dependency]"));
    assert!(dependency_matcher_source.contains("fifth_dependency"));
    assert!(dependency_matcher_source.contains("_ => {"));
    assert!(dependency_matcher_source.contains("for dependency in dependencies"));
    assert!(dependency_matcher_source.contains("if dependency == service_name"));
    assert!(dependency_matcher_source.contains("return true;"));
    assert!(dependency_matcher_source.contains("false"));
    assert!(!dependency_matcher_source.contains(".any(|dependency| dependency == service_name)"));

    assert_matcher_source_shape(
        &blocked_dependencies_source[two_dependency_matcher_index..three_dependency_matcher_index],
        "TwoServiceDependencyMatch",
        "SecondService",
    );
    assert_matcher_source_shape(
        &blocked_dependencies_source[three_dependency_matcher_index..four_dependency_matcher_index],
        "ThreeServiceDependencyMatch",
        "ThirdService",
    );
    assert_matcher_source_shape(
        &blocked_dependencies_source[four_dependency_matcher_index..five_dependency_matcher_index],
        "FourServiceDependencyMatch",
        "FourthService",
    );
    assert_matcher_source_shape(
        &blocked_dependencies_source[five_dependency_matcher_index..],
        "FiveServiceDependencyMatch",
        "FifthService",
    );

    assert_helper_uses_matcher(
        &blocked_unload_source[two_blocked_helper_index..three_blocked_helper_index],
        "first_blocked_two_service_dependency(",
        "TwoServiceDependencyMatch",
        &["FirstService", "SecondService"],
    );
    assert_helper_uses_matcher(
        &blocked_unload_source[three_blocked_helper_index..four_blocked_helper_index],
        "first_blocked_three_service_dependency(",
        "ThreeServiceDependencyMatch",
        &["FirstService", "SecondService", "ThirdService"],
    );
    assert_helper_uses_matcher(
        &blocked_unload_source[four_blocked_helper_index..five_blocked_helper_index],
        "first_blocked_four_service_dependency(",
        "FourServiceDependencyMatch",
        &[
            "FirstService",
            "SecondService",
            "ThirdService",
            "FourthService",
        ],
    );
    assert_helper_uses_matcher(
        &blocked_unload_source[five_blocked_helper_index..],
        "first_blocked_five_service_dependency(",
        "FiveServiceDependencyMatch",
        &[
            "FirstService",
            "SecondService",
            "ThirdService",
            "FourthService",
            "FifthService",
        ],
    );

    assert!(activation_tests_source.contains(
        "fn deactivate_exact_two_services_reports_first_shutdown_service_when_dependent_names_both()"
    ));
    assert!(activation_tests_source.contains(
        "fn deactivate_exact_three_services_reports_first_shutdown_service_when_dependent_names_all()"
    ));
    assert!(activation_tests_source.contains(
        "fn deactivate_exact_four_services_reports_first_shutdown_service_when_dependent_names_all()"
    ));
    assert!(activation_tests_source.contains(
        "fn deactivate_exact_five_services_reports_first_shutdown_service_when_dependent_names_all()"
    ));
}

fn assert_matcher_source_shape(source: &str, match_type: &str, last_variant: &str) {
    assert!(source.contains("match dependencies"));
    assert!(source.contains("[] => None"));
    assert!(source.contains(&format!("Some({match_type}::FirstService)")));
    assert!(source.contains(&format!("Some({match_type}::{last_variant})")));
    assert!(source.contains("[dependency]"));
    assert!(source.contains("[first_dependency, second_dependency]"));
    assert!(source.contains("[first_dependency, second_dependency, third_dependency]"));
    assert!(source
        .contains("[first_dependency, second_dependency, third_dependency, fourth_dependency]"));
    assert!(source.contains("fifth_dependency"));
    assert!(source.contains("_ => {"));
    assert!(source.contains("for dependency in dependencies"));
}

fn assert_helper_uses_matcher(
    source: &str,
    matcher_call: &str,
    match_type: &str,
    variants: &[&str],
) {
    assert!(source.contains(matcher_call));
    for variant in variants {
        assert!(source.contains(&format!("Some({match_type}::{variant})")));
    }
    assert!(!source.contains("for dependency in entry.dependencies.iter()"));
}

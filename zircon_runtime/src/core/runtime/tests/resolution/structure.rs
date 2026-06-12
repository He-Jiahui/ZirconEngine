#[test]
fn resolution_uses_registry_names_for_recursion_stack_and_dependency_walk() {
    let resolution_mod_source = include_str!("mod.rs");
    let resolution_behavior_source = include_str!("behavior.rs");
    let resolution_source = include_str!("../../handle/resolution.rs");
    let devtools_source = include_str!("../../diagnostics/devtools.rs");
    let registration_source = [
        include_str!("../../handle/registration/mod.rs"),
        include_str!("../../handle/registration/register_module.rs"),
        include_str!("../../handle/registration/descriptor_entries.rs"),
        include_str!("../../handle/registration/descriptor_entries_five.rs"),
        include_str!("../../handle/registration/descriptor_entries_four.rs"),
        include_str!("../../handle/registration/descriptor_entries_three.rs"),
        include_str!("../../handle/registration/duplicates.rs"),
        include_str!("../../handle/registration/entry.rs"),
        include_str!("../../handle/registration/service_lists.rs"),
        include_str!("../../handle/registration/validation.rs"),
    ]
    .join("\n");
    let service_entry_source = include_str!("../../state/service_entry.rs");

    assert!(resolution_mod_source.contains("mod behavior;"));
    assert!(resolution_mod_source.contains("mod structure;"));
    assert!(!resolution_mod_source.contains("#[test]"));
    assert!(!resolution_mod_source.contains("use "));
    assert!(resolution_behavior_source.contains("fn lazy_manager_is_created_on_first_resolve()"));
    assert!(resolution_behavior_source
        .contains("fn failed_lazy_manager_initialization_resets_lifecycle_and_can_retry()"));
    assert!(resolution_behavior_source
        .contains("fn failed_dependency_initialization_resets_dependent_service_and_can_retry()"));
    assert!(resolution_behavior_source
        .contains("fn resolve_exact_four_dependencies_initializes_cached_keys_directly()"));
    assert!(resolution_behavior_source
        .contains("fn resolve_exact_five_dependencies_initializes_cached_keys_directly()"));
    assert!(resolution_behavior_source
        .contains("fn four_frame_resolution_cycle_reports_canonical_registry_key()"));
    assert!(resolution_behavior_source
        .contains("fn five_frame_resolution_cycle_reports_canonical_registry_key()"));
    assert!(!resolution_behavior_source.contains("include_str!(\"../../handle/resolution.rs\")"));
    assert!(resolution_source.contains("stack: &mut Vec<RegistryName>"));
    assert!(resolution_source.contains("const RESOLUTION_STACK_FRAME_CAPACITY: usize = 1;"));
    assert!(resolution_source.contains("resolution_stack_contains(stack.as_slice(), service_key)"));
    assert!(resolution_source.contains(
        "fn resolution_stack_contains(stack: &[RegistryName], service_key: &RegistryName) -> bool",
    ));
    assert!(resolution_source.contains("[] => false"));
    assert!(resolution_source.contains("[existing] => existing == service_key"));
    assert!(resolution_source.contains("[first_existing, second_existing] => {"));
    assert!(resolution_source
        .contains("first_existing == service_key || second_existing == service_key"));
    assert!(resolution_source.contains("[first_existing, second_existing, third_existing] => {"));
    assert!(resolution_source.contains(
        "first_existing == service_key || second_existing == service_key || third_existing == service_key",
    ));
    assert!(resolution_source.contains("_ => {"));
    assert!(resolution_source.contains("for existing in stack"));
    assert!(resolution_source.contains("if existing == service_key"));
    assert!(resolution_source.contains("return true;"));
    assert!(
        !resolution_source.contains("_ => stack.iter().any(|existing| existing == service_key)")
    );
    assert!(
        resolution_source
            .matches("let mut stack = Vec::with_capacity(RESOLUTION_STACK_FRAME_CAPACITY);")
            .count()
            >= 2
    );
    assert!(resolution_source.contains("enum NamedServiceResolution"));
    assert!(resolution_source.contains("enum RegisteredServiceResolution"));
    assert!(resolution_source.contains("fn downcast_resolved_service<T: Any + Send + Sync>("));
    assert!(resolution_source.contains("match Arc::downcast::<T>(service)"));
    assert!(resolution_source.contains("fn named_service_resolution("));
    assert!(resolution_source.contains("fn registered_service_resolution("));
    assert!(resolution_source
        .contains("match self.named_service_resolution(service_name, expected_kind)?"));
    assert!(
        resolution_source.contains("NamedServiceResolution::Resolved(instance) => Ok(instance)")
    );
    assert!(resolution_source.contains("NamedServiceResolution::Pending(service_key) => {"));
    assert!(resolution_source
        .contains("match self.registered_service_resolution(service_key, expected_kind)?"));
    assert!(resolution_source
        .contains("RegisteredServiceResolution::Resolved(instance) => Ok(instance)"));
    assert!(resolution_source.contains("RegisteredServiceResolution::Pending => {"));
    assert!(resolution_source.contains(".get_key_value(service_name)"));
    assert!(resolution_source
        .contains("let Some((name, entry)) = services.get_key_value(service_name) else"));
    assert!(resolution_source.contains("fn resolve_registered_service_inner("));
    assert!(resolution_source.contains("service_key: &RegistryName"));
    assert!(
        resolution_source.contains("self.resolve_existing_service_inner(&service_key, &mut stack)")
    );
    assert!(resolution_source.contains("self.resolve_existing_service_inner(service_key, stack)"));
    assert!(resolution_source.contains(".get(service_key)"));
    assert!(resolution_source.contains("let Some(entry) = services.get(service_key) else"));
    assert!(
        resolution_source
            .matches("let Some(entry) = services.get_mut(service_key) else")
            .count()
            >= 2
    );
    assert!(resolution_source.contains("let actual_kind = name.service_kind()"));
    assert!(resolution_source.contains("let owner_module = service_key.module_name()"));
    assert!(resolution_source.contains("let canonical_service_name = service_key.as_str()"));
    assert!(resolution_source.contains(".get(owner_module)"));
    assert!(resolution_source.contains("match modules.get(owner_module)"));
    assert!(resolution_source
        .contains("Some(module) => module.lifecycle == LifecycleState::Registered"));
    assert!(resolution_source.contains("self.activate_module(owner_module)?"));
    assert!(resolution_source.contains("self.resolved_service_instance(service_key)"));
    assert!(resolution_source.contains("plugin_name: canonical_service_name.to_owned()"));
    assert!(resolution_source.contains("let (dependency_names, factory)"));
    assert!(resolution_source.contains("let factory_result = match factory"));
    assert!(resolution_source.contains("let instance = match factory_result"));
    assert!(resolution_source.contains("Err(error) => {"));
    assert!(resolution_source.contains("fn resolved_service_instance("));
    assert!(resolution_source.contains("return None;"));
    assert!(
        resolution_source
            .matches("if let Some(instance) = entry.instance.clone()")
            .count()
            >= 2
    );
    assert!(resolution_source.contains("entry.dependencies.clone()"));
    assert!(resolution_source.contains("if !dependency_names.is_empty()"));
    assert!(resolution_source
        .contains("self.resolve_dependency_services(dependency_names.as_ref(), stack)?"));
    assert!(resolution_source.contains("fn resolve_dependency_services("));
    assert!(resolution_source.contains("if let [dependency_name] = dependency_names"));
    assert!(resolution_source
        .contains("if let [first_dependency_name, second_dependency_name] = dependency_names"));
    assert!(resolution_source.contains(
        "if let [first_dependency_name, second_dependency_name, third_dependency_name] = dependency_names",
    ));
    assert!(resolution_source.contains(
        "first_dependency_name,\n            second_dependency_name,\n            third_dependency_name,\n            fourth_dependency_name,",
    ));
    assert!(resolution_source.contains("fifth_dependency_name"));
    assert!(
        resolution_source
            .matches("self.resolve_registered_service_inner(first_dependency_name, None, stack)?")
            .count()
            >= 1
    );
    assert!(
        resolution_source
            .matches("self.resolve_registered_service_inner(second_dependency_name, None, stack)?")
            .count()
            >= 1
    );
    assert!(
        resolution_source
            .matches("self.resolve_registered_service_inner(third_dependency_name, None, stack)?")
            .count()
            >= 1
    );
    assert!(
        resolution_source
            .matches("self.resolve_registered_service_inner(fourth_dependency_name, None, stack)?")
            .count()
            >= 1
    );
    assert!(
        resolution_source
            .matches("self.resolve_registered_service_inner(fifth_dependency_name, None, stack)?")
            .count()
            >= 1
    );
    assert!(resolution_source.contains("for dependency_name in dependency_names"));
    assert!(resolution_source.contains("reserve_dependency_resolution_frame(stack);"));
    assert!(resolution_source
        .contains("fn reserve_dependency_resolution_frame(stack: &mut Vec<RegistryName>)"));
    assert!(resolution_source.contains("stack.reserve(RESOLUTION_STACK_FRAME_CAPACITY)"));
    assert!(resolution_source
        .contains("self.resolve_registered_service_inner(dependency_name, None, stack)?"));
    assert!(resolution_source.contains("if result.is_err()"));
    assert!(resolution_source.contains("self.reset_initializing_service(service_key)"));
    assert!(resolution_source
        .contains("fn reset_initializing_service(&self, service_key: &RegistryName)"));
    assert!(resolution_source.contains("entry.lifecycle == LifecycleState::Initializing"));
    assert!(registration_source.contains("dependencies: &[DependencySpec]"));
    assert!(registration_source.contains("fn prepare_four_descriptor_service_entries("));
    assert!(registration_source.contains("fn prepare_five_descriptor_service_entries("));
    assert!(service_entry_source.contains("dependencies: Arc<[RegistryName]>"));
    assert!(!resolution_source.contains("stack: &mut Vec<String>"));
    assert!(!resolution_source.contains("service_key: RegistryName"));
    assert!(!resolution_source
        .contains("self.resolve_existing_service_inner(service_key.clone(), stack)"));
    assert!(!resolution_source.contains("fn resolve_named_service_inner("));
    assert!(
        !resolution_source.contains("resolution_stack_contains(stack.as_slice(), &service_key)")
    );
    assert!(!resolution_source.contains("self.resolved_service_instance(&service_key)"));
    assert!(!resolution_source.contains("Arc::downcast::<T>(service).map_err"));
    assert!(!resolution_source.contains(".ok_or_else(|| CoreError::MissingService"));
    assert!(!resolution_source.contains(".is_some_and("));
    assert!(!resolution_source.contains(".and_then(|entry| entry.instance.clone())"));
    assert!(!resolution_source.contains(".map_err(|error|"));
    assert!(!resolution_source.contains("ROOT_RESOLUTION_STACK_CAPACITY"));
    assert!(!resolution_source.contains("stack.push(service_name.to_string())"));
    assert!(!resolution_source.contains("&mut Vec::new()"));
    assert!(!resolution_source.contains("stack.iter().any(|existing| existing == &service_key)"));
    assert!(!resolution_source
        .contains("self.resolve_named_service_inner(dependency_name.as_str(), None, stack)?"));
    assert!(!resolution_source.contains(".get(service_key.as_str())"));
    assert!(!resolution_source.contains(".get_mut(service_key.as_str())"));
    assert!(!resolution_source.contains(".map(|dependency| dependency.name.clone())"));
    assert!(!resolution_source.contains(".collect::<Vec<_>>()"));
    assert!(!registration_source.contains("dependencies.clone()"));
    assert!(!resolution_source.contains("entry.name"));
    assert!(!resolution_source.contains("entry.kind"));
    assert!(!resolution_source.contains("entry.owner_module"));
    assert!(!resolution_source.contains("service_key.module_name().to_owned()"));
    assert!(!resolution_source.contains(".get(&owner_module)"));
    assert!(!resolution_source.contains("self.activate_module(&owner_module)?"));
    assert!(!resolution_source.contains("plugin_name: service_name.to_string()"));
    assert!(!resolution_source
        .contains("reserve_dependency_resolution_frame(stack, dependency_names.len())"));
    assert!(!resolution_source.contains(
        "fn reserve_dependency_resolution_frame(stack: &mut Vec<RegistryName>, dependency_count: usize)",
    ));

    let named_resolution_match_index = resolution_source
        .find("match self.named_service_resolution(service_name, expected_kind)?")
        .expect("named resolution should separate cached lookup from pending stack allocation");
    let named_resolved_index = resolution_source[named_resolution_match_index..]
        .find("NamedServiceResolution::Resolved(instance) => Ok(instance)")
        .map(|offset| named_resolution_match_index + offset)
        .expect("cached named resolution should return before stack allocation");
    let named_pending_index = resolution_source[named_resolution_match_index..]
        .find("NamedServiceResolution::Pending(service_key) => {")
        .map(|offset| named_resolution_match_index + offset)
        .expect("unresolved named services should enter the pending stack path");
    let named_stack_index = resolution_source[named_pending_index..]
        .find("let mut stack = Vec::with_capacity(RESOLUTION_STACK_FRAME_CAPACITY);")
        .map(|offset| named_pending_index + offset)
        .expect("only pending named service resolution should allocate the root stack");
    assert!(named_resolution_match_index < named_resolved_index);
    assert!(named_resolved_index < named_pending_index);
    assert!(named_pending_index < named_stack_index);

    let registered_resolution_match_index = resolution_source
        .find("match self.registered_service_resolution(service_key, expected_kind)?")
        .expect(
            "registered resolution should separate cached lookup from pending stack allocation",
        );
    let registered_resolved_index = resolution_source[registered_resolution_match_index..]
        .find("RegisteredServiceResolution::Resolved(instance) => Ok(instance)")
        .map(|offset| registered_resolution_match_index + offset)
        .expect("cached registered resolution should return before stack allocation");
    let registered_pending_index = resolution_source[registered_resolution_match_index..]
        .find("RegisteredServiceResolution::Pending => {")
        .map(|offset| registered_resolution_match_index + offset)
        .expect("unresolved registered services should enter the pending stack path");
    let registered_stack_index = resolution_source[registered_pending_index..]
        .find("let mut stack = Vec::with_capacity(RESOLUTION_STACK_FRAME_CAPACITY);")
        .map(|offset| registered_pending_index + offset)
        .expect("only pending registered service resolution should allocate the root stack");
    assert!(registered_resolution_match_index < registered_resolved_index);
    assert!(registered_resolved_index < registered_pending_index);
    assert!(registered_pending_index < registered_stack_index);

    let dependency_empty_index = resolution_source
        .find("if !dependency_names.is_empty()")
        .expect("dependency resolution should branch on non-empty dependency slices");
    let dependency_helper_call_index = resolution_source
        .find("self.resolve_dependency_services(dependency_names.as_ref(), stack)?")
        .expect("non-empty dependency slices should use the typed dependency helper");
    assert!(dependency_empty_index < dependency_helper_call_index);
    let dependency_helper_index = resolution_source
        .find("fn resolve_dependency_services(")
        .expect("dependency resolution helper should stay private to CoreHandle");
    let reserve_index = resolution_source[dependency_helper_index..]
        .find("reserve_dependency_resolution_frame(stack);")
        .map(|offset| dependency_helper_index + offset)
        .expect("non-empty dependency slices should reserve the next frame");
    let single_dependency_index = resolution_source[dependency_helper_index..]
        .find("if let [dependency_name] = dependency_names")
        .map(|offset| dependency_helper_index + offset)
        .expect("single dependency slices should bypass the multi-dependency loop");
    let two_dependency_index = resolution_source[dependency_helper_index..]
        .find("if let [first_dependency_name, second_dependency_name] = dependency_names")
        .map(|offset| dependency_helper_index + offset)
        .expect("two dependency slices should bypass the multi-dependency loop");
    let three_dependency_index = resolution_source[dependency_helper_index..]
        .find(
            "if let [first_dependency_name, second_dependency_name, third_dependency_name] = dependency_names",
        )
        .map(|offset| dependency_helper_index + offset)
        .expect("three dependency slices should bypass the multi-dependency loop");
    let four_dependency_index = resolution_source[dependency_helper_index..]
        .find("fourth_dependency_name")
        .map(|offset| dependency_helper_index + offset)
        .expect("four dependency slices should bypass the multi-dependency loop");
    let five_dependency_index = resolution_source[dependency_helper_index..]
        .find("fifth_dependency_name")
        .map(|offset| dependency_helper_index + offset)
        .expect("five dependency slices should bypass the multi-dependency loop");
    let first_two_dependency_resolve_index = resolution_source[dependency_helper_index..]
        .find("self.resolve_registered_service_inner(first_dependency_name, None, stack)?")
        .map(|offset| dependency_helper_index + offset)
        .expect("two dependency slices should resolve the first key directly");
    let second_two_dependency_resolve_index = resolution_source[dependency_helper_index..]
        .find("self.resolve_registered_service_inner(second_dependency_name, None, stack)?")
        .map(|offset| dependency_helper_index + offset)
        .expect("two dependency slices should resolve the second key directly");
    let third_dependency_resolve_index = resolution_source[dependency_helper_index..]
        .find("self.resolve_registered_service_inner(third_dependency_name, None, stack)?")
        .map(|offset| dependency_helper_index + offset)
        .expect("three dependency slices should resolve the third key directly");
    let fourth_dependency_resolve_index = resolution_source[dependency_helper_index..]
        .find("self.resolve_registered_service_inner(fourth_dependency_name, None, stack)?")
        .map(|offset| dependency_helper_index + offset)
        .expect("four dependency slices should resolve the fourth key directly");
    let fifth_dependency_resolve_index = resolution_source[dependency_helper_index..]
        .find("self.resolve_registered_service_inner(fifth_dependency_name, None, stack)?")
        .map(|offset| dependency_helper_index + offset)
        .expect("five dependency slices should resolve the fifth key directly");
    let dependency_loop_index = resolution_source[dependency_helper_index..]
        .find("for dependency_name in dependency_names")
        .map(|offset| dependency_helper_index + offset)
        .expect("multi-dependency slices should enter the dependency loop");
    assert!(reserve_index < single_dependency_index);
    assert!(single_dependency_index < two_dependency_index);
    assert!(two_dependency_index < first_two_dependency_resolve_index);
    assert!(first_two_dependency_resolve_index < second_two_dependency_resolve_index);
    assert!(second_two_dependency_resolve_index < three_dependency_index);
    assert!(three_dependency_index < third_dependency_resolve_index);
    assert!(third_dependency_resolve_index < four_dependency_index);
    assert!(four_dependency_index < fourth_dependency_resolve_index);
    assert!(fourth_dependency_resolve_index < five_dependency_index);
    assert!(five_dependency_index < fifth_dependency_resolve_index);
    assert!(fifth_dependency_resolve_index < dependency_loop_index);
    let stack_helper_index = resolution_source
        .find("fn resolution_stack_contains(")
        .expect("cycle detection should be isolated in a stack helper");
    let empty_stack_index = resolution_source[stack_helper_index..]
        .find("[] => false")
        .map(|offset| stack_helper_index + offset)
        .expect("root resolution should bypass stack iteration");
    let single_stack_index = resolution_source[stack_helper_index..]
        .find("[existing] => existing == service_key")
        .map(|offset| stack_helper_index + offset)
        .expect("single-frame resolution should compare directly");
    let two_stack_index = resolution_source[stack_helper_index..]
        .find("[first_existing, second_existing] => {")
        .map(|offset| stack_helper_index + offset)
        .expect("two-frame resolution should compare both frames directly");
    let two_stack_compare_index = resolution_source[stack_helper_index..]
        .find("first_existing == service_key || second_existing == service_key")
        .map(|offset| stack_helper_index + offset)
        .expect("two-frame resolution should avoid iterator cycle detection");
    let three_stack_index = resolution_source[stack_helper_index..]
        .find("[first_existing, second_existing, third_existing] => {")
        .map(|offset| stack_helper_index + offset)
        .expect("three-frame resolution should compare all frames directly");
    let three_stack_compare_index = resolution_source[stack_helper_index..]
        .find(
            "first_existing == service_key || second_existing == service_key || third_existing == service_key",
        )
        .map(|offset| stack_helper_index + offset)
        .expect("three-frame resolution should avoid iterator cycle detection");
    let four_stack_index = resolution_source[stack_helper_index..]
        .find("[first_existing, second_existing, third_existing, fourth_existing] => {")
        .map(|offset| stack_helper_index + offset)
        .expect("four-frame resolution should compare all frames directly");
    let four_stack_compare_index = resolution_source[stack_helper_index..]
        .find("|| fourth_existing == service_key")
        .map(|offset| stack_helper_index + offset)
        .expect("four-frame resolution should avoid iterator cycle detection");
    let five_stack_index = resolution_source[stack_helper_index..]
        .find("fifth_existing")
        .map(|offset| stack_helper_index + offset)
        .expect("five-frame resolution should compare all frames directly");
    let five_stack_compare_index = resolution_source[stack_helper_index..]
        .find("|| fifth_existing == service_key")
        .map(|offset| stack_helper_index + offset)
        .expect("five-frame resolution should avoid iterator cycle detection");
    let multi_stack_index = resolution_source[stack_helper_index..]
        .find("_ => {")
        .map(|offset| stack_helper_index + offset)
        .expect("six-or-more-frame resolution should retain fallback cycle detection");
    let multi_stack_loop_index = resolution_source[multi_stack_index..]
        .find("for existing in stack")
        .map(|offset| multi_stack_index + offset)
        .expect("six-or-more-frame resolution should scan existing stack frames directly");
    let multi_stack_match_index = resolution_source[multi_stack_loop_index..]
        .find("if existing == service_key")
        .map(|offset| multi_stack_loop_index + offset)
        .expect("six-or-more-frame resolution should compare stack frames directly");
    let multi_stack_return_index = resolution_source[multi_stack_match_index..]
        .find("return true;")
        .map(|offset| multi_stack_match_index + offset)
        .expect("six-or-more-frame resolution should return on first matching frame");
    let multi_stack_false_index = resolution_source[multi_stack_return_index..]
        .find("false")
        .map(|offset| multi_stack_return_index + offset)
        .expect("six-or-more-frame resolution should fall through to false");
    assert!(empty_stack_index < single_stack_index);
    assert!(single_stack_index < two_stack_index);
    assert!(two_stack_index < two_stack_compare_index);
    assert!(two_stack_compare_index < three_stack_index);
    assert!(three_stack_index < three_stack_compare_index);
    assert!(three_stack_compare_index < four_stack_index);
    assert!(four_stack_index < four_stack_compare_index);
    assert!(four_stack_compare_index < five_stack_index);
    assert!(five_stack_index < five_stack_compare_index);
    assert!(five_stack_compare_index < multi_stack_index);
    assert!(multi_stack_index < multi_stack_loop_index);
    assert!(multi_stack_loop_index < multi_stack_match_index);
    assert!(multi_stack_match_index < multi_stack_return_index);
    assert!(multi_stack_return_index < multi_stack_false_index);
    assert!(devtools_source.contains("owner_module: name.module_name().to_string()"));
    assert!(devtools_source.contains("kind: name.service_kind()"));
}

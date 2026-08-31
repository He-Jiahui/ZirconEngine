pub(super) fn activation_source() -> &'static str {
    include_str!("../../../handle/activation.rs")
}

pub(super) fn startup_source() -> &'static str {
    include_str!("../../../handle/activation/startup.rs")
}

pub(super) fn unload_mutation_source() -> &'static str {
    include_str!("../../../handle/activation/unload_mutation.rs")
}

pub(super) fn batch_activation_source() -> &'static str {
    include_str!("../../../handle/activation/batch.rs")
}

pub(super) fn service_lifecycle_source() -> &'static str {
    include_str!("../../../handle/activation/service_lifecycle.rs")
}

pub(super) fn service_entry_source() -> &'static str {
    include_str!("../../../state/service_entry.rs")
}

pub(super) fn reactivation_tests_source() -> &'static str {
    include_str!("../behavior/reactivation.rs")
}

pub(super) fn blocked_dependencies_root_source() -> &'static str {
    include_str!("../../../handle/activation/blocked_dependencies/mod.rs")
}

pub(super) fn blocked_dependencies_source() -> String {
    [
        include_str!("../../../handle/activation/blocked_dependencies/single.rs"),
        include_str!("../../../handle/activation/blocked_dependencies/two_service.rs"),
        include_str!("../../../handle/activation/blocked_dependencies/three_service.rs"),
        include_str!("../../../handle/activation/blocked_dependencies/four_service.rs"),
        include_str!("../../../handle/activation/blocked_dependencies/five_service.rs"),
    ]
    .join("\n")
}

pub(super) fn blocked_unload_source() -> &'static str {
    include_str!("../../../handle/activation/blocked_unload.rs")
}

pub(super) fn module_entry_source() -> &'static str {
    include_str!("../../../state/module_entry.rs")
}

pub(super) fn registration_source() -> String {
    [
        include_str!("../../../handle/registration/mod.rs"),
        include_str!("../../../handle/registration/register_module.rs"),
        include_str!("../../../handle/registration/descriptor_entries.rs"),
        include_str!("../../../handle/registration/descriptor_entries_five.rs"),
        include_str!("../../../handle/registration/descriptor_entries_four.rs"),
        include_str!("../../../handle/registration/descriptor_entries_three.rs"),
        include_str!("../../../handle/registration/duplicates.rs"),
        include_str!("../../../handle/registration/entry.rs"),
        include_str!("../../../handle/registration/service_lists/mod.rs"),
        include_str!("../../../handle/registration/service_lists/multi.rs"),
        include_str!("../../../handle/registration/service_lists/selection.rs"),
        include_str!("../../../handle/registration/service_lists/shutdown.rs"),
        include_str!("../../../handle/registration/service_lists/specialized.rs"),
        include_str!("../../../handle/registration/service_lists/types.rs"),
        include_str!("../../../handle/registration/validation.rs"),
    ]
    .join("\n")
}

pub(super) fn activation_mod_source() -> &'static str {
    include_str!("../mod.rs")
}

pub(super) fn activation_behavior_mod_source() -> &'static str {
    include_str!("../behavior.rs")
}

pub(super) fn deactivation_behavior_mod_source() -> &'static str {
    include_str!("../behavior/deactivation.rs")
}

pub(super) fn activation_tests_source() -> String {
    [
        include_str!("../behavior/activation.rs"),
        include_str!("../behavior/deactivation/blocked.rs"),
        include_str!("../behavior/deactivation/blocked/exact_five_dependency_matcher.rs"),
        include_str!("../behavior/deactivation/blocked/exact_five_without_index_map.rs"),
        include_str!("../behavior/deactivation/blocked/exact_four_dependency_matcher.rs"),
        include_str!("../behavior/deactivation/blocked/exact_two_three_dependency_matcher.rs"),
        include_str!("../behavior/deactivation/clean.rs"),
    ]
    .join("\n")
}

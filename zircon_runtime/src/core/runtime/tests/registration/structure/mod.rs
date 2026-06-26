mod behavior_layout;
mod cleanup;
mod dependency_fast_paths;
mod duplicate_detection;
mod module_layout;
mod service_count_paths;
mod service_list_caches;

pub(super) struct RegistrationStructureSources {
    pub(super) runtime_state: &'static str,
    pub(super) service_entry: &'static str,
    pub(super) registration_mod: &'static str,
    pub(super) register_module: &'static str,
    pub(super) descriptor_entries: &'static str,
    pub(super) descriptor_entries_five: &'static str,
    pub(super) descriptor_entries_four: &'static str,
    pub(super) descriptor_entries_three: &'static str,
    pub(super) duplicates: &'static str,
    pub(super) entry: &'static str,
    pub(super) service_lists: String,
    pub(super) validation: &'static str,
    pub(super) registration: String,
    pub(super) behavior_tests: String,
}

pub(super) fn registration_sources() -> RegistrationStructureSources {
    let runtime_state = include_str!("../../../state/core_runtime_state.rs");
    let service_entry = include_str!("../../../state/service_entry.rs");
    let registration_mod = include_str!("../../../handle/registration/mod.rs");
    let register_module = include_str!("../../../handle/registration/register_module.rs");
    let descriptor_entries = include_str!("../../../handle/registration/descriptor_entries.rs");
    let descriptor_entries_five =
        include_str!("../../../handle/registration/descriptor_entries_five.rs");
    let descriptor_entries_four =
        include_str!("../../../handle/registration/descriptor_entries_four.rs");
    let descriptor_entries_three =
        include_str!("../../../handle/registration/descriptor_entries_three.rs");
    let duplicates = include_str!("../../../handle/registration/duplicates.rs");
    let entry = include_str!("../../../handle/registration/entry.rs");
    let service_lists = [
        include_str!("../../../handle/registration/service_lists/mod.rs"),
        include_str!("../../../handle/registration/service_lists/multi.rs"),
        include_str!("../../../handle/registration/service_lists/shutdown.rs"),
        include_str!("../../../handle/registration/service_lists/specialized.rs"),
        include_str!("../../../handle/registration/service_lists/types.rs"),
    ]
    .join("\n");
    let validation = include_str!("../../../handle/registration/validation.rs");
    let registration = [
        registration_mod,
        register_module,
        descriptor_entries,
        descriptor_entries_five,
        descriptor_entries_four,
        descriptor_entries_three,
        service_lists.as_str(),
        entry,
        duplicates,
        validation,
    ]
    .join("\n");
    let behavior_tests = [
        include_str!("../behavior/validation.rs"),
        include_str!("../behavior/cache_lists.rs"),
        include_str!("../behavior/commit.rs"),
        include_str!("../behavior/canonical_keys.rs"),
    ]
    .join("\n");

    RegistrationStructureSources {
        runtime_state,
        service_entry,
        registration_mod,
        register_module,
        descriptor_entries,
        descriptor_entries_five,
        descriptor_entries_four,
        descriptor_entries_three,
        duplicates,
        entry,
        service_lists,
        validation,
        registration,
        behavior_tests,
    }
}

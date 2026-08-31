use super::registration_sources;

#[test]
fn registration_source_preserves_module_layout() {
    let sources = registration_sources();

    assert!(sources.registration_mod.contains("mod commit;"));
    assert!(sources.registration_mod.contains("mod descriptor_entries;"));
    assert!(sources
        .registration_mod
        .contains("mod descriptor_entries_five;"));
    assert!(sources
        .registration_mod
        .contains("mod descriptor_entries_four;"));
    assert!(sources
        .registration_mod
        .contains("mod descriptor_entries_three;"));
    assert!(sources.registration_mod.contains("mod duplicates;"));
    assert!(sources.registration_mod.contains("mod entry;"));
    assert!(sources.registration_mod.contains("mod register_module;"));
    assert!(sources.registration_mod.contains("mod service_lists;"));
    assert!(sources.registration_mod.contains("mod validation;"));
    assert!(!sources.registration_mod.contains("use "));
    assert!(!sources.registration_mod.contains("impl CoreHandle"));
    assert!(!sources.commit.contains("impl CoreHandle"));
    assert!(sources
        .commit
        .contains("pub(super) fn commit_module_registration<P>("));
    assert!(sources.commit.contains("fn assign_service_indices<'a>("));
    assert!(sources.register_module.contains("impl CoreHandle"));
    assert!(sources.register_module.contains("pub fn register_module("));
    assert!(sources
        .descriptor_entries
        .contains("pub(super) fn prepare_service_entry("));
    assert!(sources
        .descriptor_entries
        .contains("pub(super) fn prepare_driver_entry("));
    assert!(sources
        .descriptor_entries
        .contains("pub(super) fn prepare_manager_entry("));
    assert!(sources
        .descriptor_entries
        .contains("pub(super) fn prepare_plugin_entry("));
    assert!(!sources
        .descriptor_entries_three
        .contains("fn prepare_driver_entry("));
    assert!(!sources
        .descriptor_entries_three
        .contains("fn prepare_manager_entry("));
    assert!(!sources
        .descriptor_entries_three
        .contains("fn prepare_plugin_entry("));
    assert!(!sources
        .descriptor_entries_four
        .contains("fn prepare_driver_entry("));
    assert!(!sources
        .descriptor_entries_four
        .contains("fn prepare_manager_entry("));
    assert!(!sources
        .descriptor_entries_four
        .contains("fn prepare_plugin_entry("));
    assert!(!sources
        .descriptor_entries_five
        .contains("fn prepare_driver_entry("));
    assert!(!sources
        .descriptor_entries_five
        .contains("fn prepare_manager_entry("));
    assert!(!sources
        .descriptor_entries_five
        .contains("fn prepare_plugin_entry("));
    assert!(sources
        .descriptor_entries_three
        .contains("pub(super) fn prepare_three_descriptor_service_entries("));
    assert!(sources
        .descriptor_entries_four
        .contains("pub(super) fn prepare_four_descriptor_service_entries("));
    assert!(sources
        .descriptor_entries_five
        .contains("pub(super) fn prepare_five_descriptor_service_entries("));
    assert!(sources
        .duplicates
        .contains("pub(super) fn duplicate_existing_pending_service_name"));
    assert!(sources.entry.contains("pub(super) fn service_entry("));
    assert!(sources
        .service_lists
        .contains("pub(in crate::core::runtime::handle::registration) struct ModuleServiceLists"));
    assert!(sources
        .validation
        .contains("pub(super) fn validate_service_descriptor("));
    assert!(sources
        .runtime_state
        .contains("HashMap<RegistryName, ServiceEntry>"));
    assert!(sources
        .service_entry
        .contains("dependencies: Arc<[RegistryName]>"));
}

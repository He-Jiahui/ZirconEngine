use super::builtin_template_bindings;

#[test]
fn builtin_template_binding_registry_is_process_cached() {
    let first = builtin_template_bindings();
    let second = builtin_template_bindings();

    assert!(std::ptr::eq(first, second));
    assert!(first.contains_key("WorkbenchMenuBar/OpenProject"));
}

#[test]
fn componentized_document_tabs_share_the_canonical_dynamic_payloads() {
    let bindings = builtin_template_bindings();

    for (workbench_id, canonical_id) in [
        ("Workbench/ActivateDocumentTab", "DocumentTabs/ActivateTab"),
        ("Workbench/CloseDocumentTab", "DocumentTabs/CloseTab"),
    ] {
        let workbench = bindings
            .get(workbench_id)
            .unwrap_or_else(|| panic!("{workbench_id} should be registered"));
        let canonical = bindings
            .get(canonical_id)
            .unwrap_or_else(|| panic!("{canonical_id} should be registered"));
        assert_eq!(workbench.path().event_kind, canonical.path().event_kind);
        assert_eq!(workbench.payload(), canonical.payload());
    }
}

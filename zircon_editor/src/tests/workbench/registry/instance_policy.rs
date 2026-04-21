use crate::ui::workbench::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind, ViewRegistry,
};

#[test]
fn view_registry_reuses_single_instance_and_allows_multi_instance() {
    let mut registry = ViewRegistry::default();
    registry
        .register_view(ViewDescriptor::new(
            ViewDescriptorId::new("editor.hierarchy"),
            ViewKind::ActivityView,
            "Hierarchy",
        ))
        .unwrap();
    registry
        .register_view(
            ViewDescriptor::new(
                ViewDescriptorId::new("editor.prefab"),
                ViewKind::ActivityWindow,
                "Prefab Editor",
            )
            .with_multi_instance(true)
            .with_preferred_host(PreferredHost::DocumentCenter),
        )
        .unwrap();

    let first = registry
        .open_descriptor(ViewDescriptorId::new("editor.hierarchy"))
        .unwrap();
    let second = registry
        .open_descriptor(ViewDescriptorId::new("editor.hierarchy"))
        .unwrap();
    let prefab_a = registry
        .open_descriptor(ViewDescriptorId::new("editor.prefab"))
        .unwrap();
    let prefab_b = registry
        .open_descriptor(ViewDescriptorId::new("editor.prefab"))
        .unwrap();

    assert_eq!(first.instance_id, second.instance_id);
    assert_ne!(prefab_a.instance_id, prefab_b.instance_id);
}

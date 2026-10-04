use super::*;
use crate::ui::workbench::view::ViewKind;

#[test]
fn unregister_view_removes_an_unreferenced_descriptor() {
    let mut registry = ViewRegistry::default();
    let descriptor = ViewDescriptor::new(
        ViewDescriptorId::new("plugin.example.panel"),
        ViewKind::ActivityView,
        "Panel",
    );
    registry.register_view(descriptor.clone()).unwrap();

    assert_eq!(
        registry.unregister_view(&descriptor.descriptor_id),
        Ok(descriptor)
    );
    assert!(registry
        .descriptor(&ViewDescriptorId::new("plugin.example.panel"))
        .is_none());
}

#[test]
fn unregister_view_rejects_a_live_instance() {
    let mut registry = ViewRegistry::default();
    let descriptor = ViewDescriptor::new(
        ViewDescriptorId::new("plugin.example.panel"),
        ViewKind::ActivityView,
        "Panel",
    );
    registry.register_view(descriptor.clone()).unwrap();
    registry
        .open_descriptor(descriptor.descriptor_id.clone())
        .unwrap();

    let error = registry
        .unregister_view(&descriptor.descriptor_id)
        .expect_err("live instances must keep their descriptor alive");
    assert!(error.contains("instances are open"));
}

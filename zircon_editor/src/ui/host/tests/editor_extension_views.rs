use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind, ViewRegistry};

use super::validate_extension_view_descriptors;

fn view(id: &str) -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new(id),
        ViewKind::ActivityView,
        format!("View {id}"),
    )
}

#[test]
fn borrowed_view_validation_accepts_unique_ids() {
    let registry = ViewRegistry::default();
    let views = [view("plugin.example.first"), view("plugin.example.second")];

    validate_extension_view_descriptors(&registry, &views).unwrap();
}

#[test]
fn borrowed_view_validation_rejects_a_batch_duplicate() {
    let registry = ViewRegistry::default();
    let views = [view("plugin.example.same"), view("plugin.example.same")];

    assert!(validate_extension_view_descriptors(&registry, &views).is_err());
}

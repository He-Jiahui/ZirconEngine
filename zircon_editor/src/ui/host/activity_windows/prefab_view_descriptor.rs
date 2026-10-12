use crate::core::commands::DocumentKind;
use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind, WorkbenchSlot};

pub(super) fn prefab_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.prefab"),
        ViewKind::ActivityWindow,
        "Prefab Editor",
    )
    .with_document_kind(DocumentKind::prefab())
    .with_multi_instance(true)
    .with_workbench_slot(WorkbenchSlot::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::PrefabEditor,
    ))
    .with_icon_key("prefab")
}

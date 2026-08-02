use crate::core::commands::DocumentKind;
use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind, WorkbenchSlot};

pub(super) fn ensure_ui_asset_descriptor(descriptors: &mut Vec<ViewDescriptor>) {
    if descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.ui_asset")
    {
        return;
    }

    descriptors.push(
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.ui_asset"),
            ViewKind::ActivityWindow,
            "UI Asset Editor",
        )
        .with_document_kind(DocumentKind::ui_asset())
        .with_multi_instance(true)
        .with_workbench_slot(WorkbenchSlot::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::UiAssetEditor,
        ))
        .with_icon_key("ui-asset"),
    );
}

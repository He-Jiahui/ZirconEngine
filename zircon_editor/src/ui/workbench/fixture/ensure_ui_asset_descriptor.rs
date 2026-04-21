use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind};

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
        .with_multi_instance(true)
        .with_preferred_host(PreferredHost::DocumentCenter)
        .with_default_constraints(default_constraints_for_content(
            ViewContentKind::UiAssetEditor,
        ))
        .with_icon_key("ui-asset"),
    );
}

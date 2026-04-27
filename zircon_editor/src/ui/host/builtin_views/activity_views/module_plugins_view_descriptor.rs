use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind};

pub(super) fn module_plugins_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.module_plugins"),
        ViewKind::ActivityView,
        "Modules",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::LeftBottom)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::ModulePlugins,
    ))
    .with_icon_key("module-plugins")
}

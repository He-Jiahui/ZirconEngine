use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::WorkbenchSlot;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind};

pub(super) fn assets_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.assets"),
        ViewKind::ActivityView,
        "Assets",
    )
    .with_workbench_slot(WorkbenchSlot::LeftTopDrawer)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Assets))
    .with_icon_key("assets")
}

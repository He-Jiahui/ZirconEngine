use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind};

pub(super) fn inspector_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.inspector"),
        ViewKind::ActivityView,
        "Inspector",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::RightTop)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Inspector))
    .with_icon_key("inspector")
}

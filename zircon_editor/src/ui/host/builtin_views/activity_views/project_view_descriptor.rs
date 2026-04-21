use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind};

pub(super) fn project_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.project"),
        ViewKind::ActivityView,
        "Project",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Project))
    .with_icon_key("project")
}

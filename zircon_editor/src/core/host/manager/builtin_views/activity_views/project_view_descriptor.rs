use crate::default_constraints_for_content;
use crate::layout::ActivityDrawerSlot;
use crate::view::{ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

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

use crate::default_constraints_for_content;
use crate::layout::ActivityDrawerSlot;
use crate::view::{ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

pub(super) fn hierarchy_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.hierarchy"),
        ViewKind::ActivityView,
        "Hierarchy",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Hierarchy))
    .with_icon_key("hierarchy")
}

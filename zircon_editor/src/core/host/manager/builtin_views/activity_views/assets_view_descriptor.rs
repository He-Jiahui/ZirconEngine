use crate::default_constraints_for_content;
use crate::layout::ActivityDrawerSlot;
use crate::view::{ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

pub(super) fn assets_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.assets"),
        ViewKind::ActivityView,
        "Assets",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Assets))
    .with_icon_key("assets")
}

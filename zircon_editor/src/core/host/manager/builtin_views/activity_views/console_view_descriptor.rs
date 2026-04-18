use crate::default_constraints_for_content;
use crate::layout::ActivityDrawerSlot;
use crate::view::{ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

pub(super) fn console_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.console"),
        ViewKind::ActivityView,
        "Console",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::BottomLeft)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Console))
    .with_icon_key("console")
}

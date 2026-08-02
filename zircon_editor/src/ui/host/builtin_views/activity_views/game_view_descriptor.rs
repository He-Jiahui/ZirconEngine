use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    DockPolicy, ViewDescriptor, ViewDescriptorId, ViewKind, WorkbenchSlot,
};

pub(super) fn game_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.game"),
        ViewKind::ActivityView,
        "Game",
    )
    .with_dock_policy(DockPolicy::DrawerOrDocument)
    .with_workbench_slot(WorkbenchSlot::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Game))
    .with_icon_key("game")
}

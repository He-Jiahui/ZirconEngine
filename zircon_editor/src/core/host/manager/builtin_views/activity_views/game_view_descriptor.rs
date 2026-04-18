use crate::default_constraints_for_content;
use crate::view::{DockPolicy, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

pub(super) fn game_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.game"),
        ViewKind::ActivityView,
        "Game",
    )
    .with_dock_policy(DockPolicy::DrawerOrDocument)
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Game))
    .with_icon_key("game")
}

use crate::default_constraints_for_content;
use crate::view::{DockPolicy, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

pub(super) fn scene_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.scene"),
        ViewKind::ActivityView,
        "Scene",
    )
    .with_dock_policy(DockPolicy::DrawerOrDocument)
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Scene))
    .with_icon_key("scene")
}

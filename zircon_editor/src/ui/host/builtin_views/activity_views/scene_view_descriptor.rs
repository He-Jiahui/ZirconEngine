use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    DockPolicy, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
};

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

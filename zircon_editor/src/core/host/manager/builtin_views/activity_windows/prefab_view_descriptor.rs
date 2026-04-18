use crate::default_constraints_for_content;
use crate::view::{PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind};
use crate::ViewContentKind;

pub(super) fn prefab_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.prefab"),
        ViewKind::ActivityWindow,
        "Prefab Editor",
    )
    .with_multi_instance(true)
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::PrefabEditor,
    ))
    .with_icon_key("prefab")
}

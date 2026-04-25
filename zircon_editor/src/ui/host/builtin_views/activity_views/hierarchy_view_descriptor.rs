use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn hierarchy_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.hierarchy"),
        ViewKind::ActivityView,
        "Hierarchy",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::LeftTop)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Hierarchy))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "pane.hierarchy.body",
        PanePayloadKind::HierarchyV1,
        PaneRouteNamespace::Selection,
        PaneInteractionMode::HybridNativeSlot,
    )))
    .with_icon_key("hierarchy")
}

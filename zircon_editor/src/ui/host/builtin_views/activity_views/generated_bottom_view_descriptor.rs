use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn generated_bottom_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.generated_bottom"),
        ViewKind::ActivityView,
        "Generated Output",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::Bottom)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::GeneratedBottom,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "res://ui/editor/host/generated_bottom_body.zui",
        PanePayloadKind::GeneratedBottomV1,
        PaneRouteNamespace::Dock,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("generated-output")
}

use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn console_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.console"),
        ViewKind::ActivityView,
        "Console",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::Bottom)
    .with_default_constraints(default_constraints_for_content(ViewContentKind::Console))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "pane.console.body",
        PanePayloadKind::ConsoleV1,
        PaneRouteNamespace::Dock,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("console")
}

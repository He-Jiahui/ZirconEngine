use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn runtime_diagnostics_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.runtime_diagnostics"),
        ViewKind::ActivityView,
        "Runtime Diagnostics",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::BottomRight)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::RuntimeDiagnostics,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "pane.runtime.diagnostics.body",
        PanePayloadKind::RuntimeDiagnosticsV1,
        PaneRouteNamespace::Diagnostics,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("runtime-diagnostics")
}

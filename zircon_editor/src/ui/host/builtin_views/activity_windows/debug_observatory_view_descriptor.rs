use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind, WorkbenchSlot,
};

pub(super) fn debug_observatory_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.debug_observatory"),
        ViewKind::ActivityWindow,
        "Debug Observatory",
    )
    .with_workbench_slot(WorkbenchSlot::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::RuntimeDiagnostics,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "res://ui/editor/host/runtime_diagnostics_body.zui",
        PanePayloadKind::RuntimeDiagnosticsV1,
        PaneRouteNamespace::Diagnostics,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("debug-observatory")
}

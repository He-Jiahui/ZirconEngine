use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn debug_observatory_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.debug_observatory"),
        ViewKind::ActivityWindow,
        "Debug Observatory",
    )
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::RuntimeDiagnostics,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "pane.runtime.diagnostics.body",
        PanePayloadKind::RuntimeDiagnosticsV1,
        PaneRouteNamespace::Diagnostics,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("debug-observatory")
}

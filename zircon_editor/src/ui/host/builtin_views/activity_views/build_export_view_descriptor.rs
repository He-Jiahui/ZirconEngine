use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn build_export_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.build_export_desktop"),
        ViewKind::ActivityView,
        "Desktop Export",
    )
    .with_preferred_drawer_slot(ActivityDrawerSlot::Bottom)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::BuildExport,
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "pane.build_export_desktop.body",
        PanePayloadKind::BuildExportV1,
        PaneRouteNamespace::Diagnostics,
        PaneInteractionMode::HybridNativeSlot,
    )))
    .with_icon_key("build-export")
}

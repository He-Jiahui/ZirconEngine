use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PaneBodySpec, PaneInteractionMode, PanePayloadKind,
    PaneRouteNamespace, PaneTemplateSpec, ViewDescriptor, ViewDescriptorId, ViewKind,
    WorkbenchSlot,
};

pub(super) fn material_demo_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.material_demo_window"),
        ViewKind::ActivityWindow,
        "Material Demo Window",
    )
    .with_workbench_slot(WorkbenchSlot::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::UiComponentShowcase,
    ))
    .with_activity_window_template(ActivityWindowTemplateSpec::new(
        "res://ui/editor/material_demo_window.zui",
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "res://ui/editor/material_demo_window.zui",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("material-demo")
}

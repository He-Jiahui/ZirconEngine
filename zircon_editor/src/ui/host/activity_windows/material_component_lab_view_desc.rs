use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PaneBodySpec, PaneInteractionMode, PanePayloadKind,
    PaneRouteNamespace, PaneTemplateSpec, ViewDescriptor, ViewDescriptorId, ViewKind,
    WorkbenchSlot,
};

pub(super) fn material_component_lab_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.material_component_lab"),
        ViewKind::ActivityWindow,
        "Material Component Lab",
    )
    .with_workbench_slot(WorkbenchSlot::ExclusiveMainPage)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::UiComponentShowcase,
    ))
    .with_activity_window_template(ActivityWindowTemplateSpec::new(
        "res://ui/editor/material_component_lab.zui",
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "res://ui/editor/material_component_lab.zui",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("material-component-lab")
}

use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PaneBodySpec, PaneInteractionMode, PanePayloadKind,
    PaneRouteNamespace, PaneTemplateSpec, PreferredHost, ViewDescriptor, ViewDescriptorId,
    ViewKind,
};

pub(super) fn material_component_lab_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.material_component_lab"),
        ViewKind::ActivityWindow,
        "Material Component Lab",
    )
    .with_preferred_host(PreferredHost::ExclusiveMainPage)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::UiComponentShowcase,
    ))
    .with_activity_window_template(ActivityWindowTemplateSpec::new(
        "editor.window.material_component_lab",
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "editor.window.material_component_lab",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("material-component-lab")
}

use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PaneBodySpec, PaneInteractionMode, PanePayloadKind,
    PaneRouteNamespace, PaneTemplateSpec, PreferredHost, ViewDescriptor, ViewDescriptorId,
    ViewKind,
};

pub(super) fn component_showcase_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.ui_component_showcase"),
        ViewKind::ActivityWindow,
        "UI Component Showcase",
    )
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::UiComponentShowcase,
    ))
    .with_activity_window_template(ActivityWindowTemplateSpec::new(
        "editor.window.ui_component_showcase",
    ))
    .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
        "editor.window.ui_component_showcase",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    )))
    .with_icon_key("ui-components")
}

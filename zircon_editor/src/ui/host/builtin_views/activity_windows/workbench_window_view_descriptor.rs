use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
};

pub(super) fn workbench_window_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new("editor.workbench_window"),
        ViewKind::ActivityWindow,
        "Workbench",
    )
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(
        crate::ui::workbench::autolayout::default_constraints_for_content(ViewContentKind::Scene),
    )
    .with_activity_window_template(ActivityWindowTemplateSpec::new("editor.window.workbench"))
    .with_icon_key("workbench")
}
